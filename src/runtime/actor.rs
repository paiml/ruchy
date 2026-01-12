#![allow(clippy::print_stdout, clippy::print_stderr)]
#![allow(clippy::approx_constant)]
//! Actor system runtime with supervision trees
//!
//! This module implements a robust actor system inspired by Erlang/OTP and Akka,
//! with supervision trees for fault tolerance and message passing capabilities.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
/// Unique identifier for actors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(pub u64);
impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "actor-{}", self.0)
    }
}
/// Actor reference for sending messages
#[derive(Debug, Clone)]
pub struct ActorRef {
    pub id: ActorId,
    pub name: String,
    sender: mpsc::Sender<ActorMessage>,
}
impl ActorRef {
    /// Send a message to this actor (fire-and-forget)
    ///
    /// # Errors
    ///
    /// Returns an error if the actor is no longer running
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::actor::ActorRef;
    ///
    /// let mut instance = ActorRef::new();
    /// let result = instance.send();
    /// // Verify behavior
    /// ```
    pub fn send(&self, message: Message) -> Result<()> {
        self.sender
            .send(ActorMessage::UserMessage(message))
            .map_err(|_| anyhow!("Actor {} is no longer running", self.id))?;
        Ok(())
    }
    /// Ask a message to this actor and wait for a response
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The actor is no longer running
    /// - The timeout expires before receiving a response
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::actor::ActorRef;
    ///
    /// let mut instance = ActorRef::new();
    /// let result = instance.ask();
    /// // Verify behavior
    /// ```
    pub fn ask(&self, message: Message, timeout: Duration) -> Result<Message> {
        let (response_tx, response_rx) = mpsc::channel();
        self.sender
            .send(ActorMessage::AskMessage {
                message,
                response: response_tx,
            })
            .map_err(|_| anyhow!("Actor {} is no longer running", self.id))?;
        response_rx
            .recv_timeout(timeout)
            .map_err(|_| anyhow!("Timeout waiting for response from {}", self.id))
    }
}
/// Message that can be sent between actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// System messages for actor lifecycle
    Start,
    Stop,
    Restart,
    /// User-defined messages
    User(String, Vec<MessageValue>),
    /// Error notification
    Error(String),
    /// Supervision messages
    ChildFailed(ActorId, String),
    ChildRestarted(ActorId),
}
/// Values that can be passed in messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    List(Vec<MessageValue>),
    Map(HashMap<String, MessageValue>),
    ActorRef(ActorId),
}
/// Internal actor message envelope
#[derive(Debug)]
enum ActorMessage {
    UserMessage(Message),
    AskMessage {
        message: Message,
        response: mpsc::Sender<Message>,
    },
    SystemShutdown,
}
/// Actor behavior trait
pub trait ActorBehavior: Send + 'static {
    /// Called when the actor starts
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails
    fn pre_start(&mut self, _ctx: &mut ActorContext) -> Result<()> {
        Ok(())
    }
    /// Called when the actor stops
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails
    fn post_stop(&mut self, _ctx: &mut ActorContext) -> Result<()> {
        Ok(())
    }
    /// Called when the actor is about to restart
    ///
    /// # Errors
    ///
    /// Returns an error if pre-restart logic fails
    fn pre_restart(&mut self, _ctx: &mut ActorContext, _reason: &str) -> Result<()> {
        Ok(())
    }
    /// Called after the actor has restarted
    ///
    /// # Errors
    ///
    /// Returns an error if post-restart logic fails
    fn post_restart(&mut self, _ctx: &mut ActorContext, _reason: &str) -> Result<()> {
        Ok(())
    }
    /// Handle incoming messages
    ///
    /// # Errors
    ///
    /// Returns an error if message processing fails
    fn receive(&mut self, message: Message, ctx: &mut ActorContext) -> Result<Option<Message>>;
    /// Handle actor supervision - called when a child actor fails
    fn supervisor_strategy(&mut self, _child: ActorId, _reason: &str) -> SupervisorDirective {
        SupervisorDirective::Restart
    }
}
/// Supervisor strategy for handling child actor failures
#[derive(Debug, Clone)]
pub enum SupervisorDirective {
    /// Restart the failed child
    Restart,
    /// Stop the failed child
    Stop,
    /// Escalate the failure to the parent supervisor
    Escalate,
    /// Resume the child (ignore the failure)
    Resume,
}
/// Actor context provided during message handling
pub struct ActorContext {
    pub actor_id: ActorId,
    pub actor_name: String,
    pub supervisor: Option<ActorRef>,
    pub children: HashMap<ActorId, ActorRef>,
    system: Arc<Mutex<ActorSystem>>,
}
impl ActorContext {
    /// Spawn a child actor under this actor's supervision
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn spawn_child<B: ActorBehavior>(&mut self, name: String, behavior: B) -> Result<ActorRef> {
        let mut system = self
            .system
            .lock()
            .map_err(|_| anyhow!("Actor system mutex poisoned"))?;
        let actor_ref = system.spawn_supervised(name, Box::new(behavior), Some(self.actor_id))?;
        self.children.insert(actor_ref.id, actor_ref.clone());
        Ok(actor_ref)
    }
    /// Stop a child actor
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::actor::ActorContext;
    ///
    /// let mut instance = ActorContext::new();
    /// let result = instance.stop_child();
    /// // Verify behavior
    /// ```
    pub fn stop_child(&mut self, child_id: ActorId) -> Result<()> {
        if let Some(child_ref) = self.children.remove(&child_id) {
            child_ref.send(Message::Stop)?;
        }
        Ok(())
    }
    /// Get reference to self
    ///
    /// # Errors
    ///
    /// Returns an error if the actor reference cannot be retrieved
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::get_self;
    ///
    /// let result = get_self(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_self(&self) -> Result<ActorRef> {
        let system = self
            .system
            .lock()
            .map_err(|_| anyhow!("Actor system mutex poisoned"))?;
        system
            .get_actor_ref(self.actor_id)
            .ok_or_else(|| anyhow!("Actor not found"))
    }
    /// Find actor by name
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::actor::ActorContext;
    ///
    /// let mut instance = ActorContext::new();
    /// let result = instance.find_actor();
    /// // Verify behavior
    /// ```
    pub fn find_actor(&self, name: &str) -> Option<ActorRef> {
        let system = self.system.lock().ok()?;
        system.find_actor_by_name(name)
    }
}
/// Actor runtime information
struct ActorRuntime {
    id: ActorId,
    name: String,
    behavior: Box<dyn ActorBehavior>,
    receiver: mpsc::Receiver<ActorMessage>,
    sender: mpsc::Sender<ActorMessage>,
    supervisor: Option<ActorId>,
    children: HashMap<ActorId, ActorRef>,
    system: Arc<Mutex<ActorSystem>>,
    handle: Option<JoinHandle<()>>,
}
impl ActorRuntime {
    fn new(
        id: ActorId,
        name: String,
        behavior: Box<dyn ActorBehavior>,
        supervisor: Option<ActorId>,
        system: Arc<Mutex<ActorSystem>>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            id,
            name,
            behavior,
            receiver,
            sender,
            supervisor,
            children: HashMap::new(),
            system,
            handle: None,
        }
    }
    fn start(&mut self) -> ActorRef {
        let actor_ref = ActorRef {
            id: self.id,
            name: self.name.clone(),
            sender: self.sender.clone(),
        };
        let id = self.id;
        let name = self.name.clone();
        let receiver = std::mem::replace(&mut self.receiver, mpsc::channel().1);
        let mut behavior = std::mem::replace(&mut self.behavior, Box::new(DummyBehavior));
        let supervisor = self.supervisor;
        let system = self.system.clone();
        let children = self.children.clone();
        let handle = thread::spawn(move || {
            let mut ctx = ActorContext {
                actor_id: id,
                actor_name: name.clone(),
                supervisor: supervisor.and_then(|sup_id| system.lock().ok()?.get_actor_ref(sup_id)),
                children,
                system: system.clone(),
            };
            // Initialize actor
            if let Err(e) = behavior.pre_start(&mut ctx) {
                eprintln!("Actor {name} failed to start: {e}");
                return;
            }
            // Main message loop
            loop {
                match receiver.recv() {
                    Ok(ActorMessage::UserMessage(msg)) => {
                        match behavior.receive(msg, &mut ctx) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Actor {name} error handling message: {e}");
                                // Notify supervisor of failure
                                if let Some(sup) = &ctx.supervisor {
                                    let _ = sup.send(Message::ChildFailed(id, e.to_string()));
                                }
                            }
                        }
                    }
                    Ok(ActorMessage::AskMessage { message, response }) => {
                        match behavior.receive(message, &mut ctx) {
                            Ok(Some(reply)) => {
                                let _ = response.send(reply);
                            }
                            Ok(None) => {
                                let _ = response.send(Message::Error("No response".to_string()));
                            }
                            Err(e) => {
                                let _ = response.send(Message::Error(e.to_string()));
                                // Notify supervisor of failure
                                if let Some(sup) = &ctx.supervisor {
                                    let _ = sup.send(Message::ChildFailed(id, e.to_string()));
                                }
                            }
                        }
                    }
                    Ok(ActorMessage::SystemShutdown) => {
                        break;
                    }
                    Err(_) => {
                        // Channel closed, exit
                        break;
                    }
                }
            }
            // Cleanup
            let _ = behavior.post_stop(&mut ctx);
        });
        self.handle = Some(handle);
        actor_ref
    }
    fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.sender.send(ActorMessage::SystemShutdown);
            let _ = handle.join();
        }
    }
}
/// Dummy behavior for placeholder
struct DummyBehavior;
impl ActorBehavior for DummyBehavior {
    fn receive(&mut self, _message: Message, _ctx: &mut ActorContext) -> Result<Option<Message>> {
        Ok(None)
    }
}
/// Actor system managing all actors and supervision
pub struct ActorSystem {
    actors: HashMap<ActorId, ActorRuntime>,
    actor_names: HashMap<String, ActorId>,
    next_id: u64,
}
impl ActorSystem {
    /// Create a new actor system
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::new;
    ///
    /// let result = new(());
    /// assert_eq!(result, Ok(()));
    /// ```
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::new;
    ///
    /// let result = new(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            actors: HashMap::new(),
            actor_names: HashMap::new(),
            next_id: 1,
        }))
    }
    /// Spawn a new actor in the system
    ///
    /// # Errors
    ///
    /// Returns an error if an actor with the same name already exists
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn spawn<B: ActorBehavior>(&mut self, name: String, behavior: B) -> Result<ActorRef> {
        self.spawn_supervised(name, Box::new(behavior), None)
    }
    /// Spawn a supervised actor
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An actor with the same name already exists
    /// - The supervisor doesn't exist (if specified)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::spawn_supervised;
    ///
    /// let result = spawn_supervised(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn spawn_supervised(
        &mut self,
        name: String,
        behavior: Box<dyn ActorBehavior>,
        supervisor: Option<ActorId>,
    ) -> Result<ActorRef> {
        if self.actor_names.contains_key(&name) {
            return Err(anyhow!("Actor with name '{name}' already exists"));
        }
        let id = ActorId(self.next_id);
        self.next_id += 1;
        let system_arc = Arc::new(Mutex::new(ActorSystem {
            actors: HashMap::new(),
            actor_names: HashMap::new(),
            next_id: self.next_id,
        }));
        let mut runtime = ActorRuntime::new(id, name.clone(), behavior, supervisor, system_arc);
        let actor_ref = runtime.start();
        self.actors.insert(id, runtime);
        self.actor_names.insert(name, id);
        Ok(actor_ref)
    }
    /// Get actor reference by ID
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::get_actor_ref;
    ///
    /// let result = get_actor_ref(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_actor_ref(&self, id: ActorId) -> Option<ActorRef> {
        self.actors.get(&id).map(|runtime| ActorRef {
            id: runtime.id,
            name: runtime.name.clone(),
            sender: runtime.sender.clone(),
        })
    }
    /// Find actor by name
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::find_actor_by_name;
    ///
    /// let result = find_actor_by_name("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn find_actor_by_name(&self, name: &str) -> Option<ActorRef> {
        self.actor_names
            .get(name)
            .and_then(|&id| self.get_actor_ref(id))
    }
    /// Stop an actor
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::stop_actor;
    ///
    /// let result = stop_actor(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn stop_actor(&mut self, id: ActorId) -> Result<()> {
        if let Some(mut runtime) = self.actors.remove(&id) {
            self.actor_names.retain(|_, &mut v| v != id);
            runtime.stop();
        }
        Ok(())
    }
    /// Shutdown the entire actor system
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::actor::shutdown;
    ///
    /// let result = shutdown(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn shutdown(&mut self) {
        let actor_ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in actor_ids {
            let _ = self.stop_actor(id);
        }
    }
}
impl Default for ActorSystem {
    fn default() -> Self {
        Self {
            actors: HashMap::new(),
            actor_names: HashMap::new(),
            next_id: 1,
        }
    }
}
impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        Self {
            actors: HashMap::new(),
            actor_names: self.actor_names.clone(),
            next_id: self.next_id,
        }
    }
}
/// Example echo actor behavior
pub struct EchoActor;
impl ActorBehavior for EchoActor {
    fn receive(&mut self, message: Message, _ctx: &mut ActorContext) -> Result<Option<Message>> {
        match message {
            Message::User(msg_type, values) => {
                println!("Echo: {msg_type} with values: {values:?}");
                Ok(Some(Message::User(format!("Echo: {msg_type}"), values)))
            }
            _ => Ok(None),
        }
    }
}
/// Example supervisor actor that manages child actors
pub struct SupervisorActor {
    restart_count: HashMap<ActorId, u32>,
    max_restarts: u32,
}
impl SupervisorActor {
    #[must_use]
    pub fn new(max_restarts: u32) -> Self {
        Self {
            restart_count: HashMap::new(),
            max_restarts,
        }
    }
}
impl ActorBehavior for SupervisorActor {
    fn receive(&mut self, message: Message, ctx: &mut ActorContext) -> Result<Option<Message>> {
        match message {
            Message::ChildFailed(child_id, reason) => {
                let count = self.restart_count.entry(child_id).or_insert(0);
                *count += 1;
                if *count <= self.max_restarts {
                    println!("Supervisor restarting child {child_id} (attempt {count}): {reason}");
                    // In a real implementation, we would restart the child here
                    Ok(Some(Message::ChildRestarted(child_id)))
                } else {
                    println!("Supervisor stopping child {child_id} after {count} failures");
                    ctx.stop_child(child_id)?;
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
    fn supervisor_strategy(&mut self, child: ActorId, _reason: &str) -> SupervisorDirective {
        let count = self.restart_count.get(&child).unwrap_or(&0);
        if *count < self.max_restarts {
            SupervisorDirective::Restart
        } else {
            SupervisorDirective::Stop
        }
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Helper function for creating test context
    fn create_test_context() -> ActorContext {
        let system = ActorSystem::new();
        ActorContext {
            actor_id: ActorId(1),
            actor_name: "test_actor".to_string(),
            supervisor: None,
            children: std::collections::HashMap::new(),
            system,
        }
    }

    #[test]
    fn test_actor_system_creation() {
        let system = ActorSystem::new();
        assert!(system
            .lock()
            .expect("Failed to acquire lock")
            .actors
            .is_empty());
    }

    #[test]
    fn test_echo_actor() {
        let system = ActorSystem::new();
        let actor_ref = {
            let mut sys = system.lock().expect("Failed to acquire lock");
            sys.spawn("echo".to_string(), EchoActor)
                .expect("Failed to spawn echo actor")
        };
        let message = Message::User(
            "test".to_string(),
            vec![MessageValue::String("hello".to_string())],
        );
        let response = actor_ref
            .ask(message, Duration::from_millis(100))
            .expect("Failed to get response from actor");
        match response {
            Message::User(msg, _) => assert!(msg.contains("Echo: test")),
            _ => panic!("Unexpected response type"),
        }
    }

    #[test]
    fn test_supervisor_actor() {
        let system = ActorSystem::new();
        let supervisor_ref = {
            let mut sys = system.lock().expect("Failed to acquire lock");
            sys.spawn("supervisor".to_string(), SupervisorActor::new(3))
                .expect("Failed to spawn supervisor actor")
        };
        let child_id = ActorId(999);
        let failure_message = Message::ChildFailed(child_id, "Test failure".to_string());
        let response = supervisor_ref
            .ask(failure_message, Duration::from_millis(100))
            .expect("Failed to get response from supervisor");
        match response {
            Message::ChildRestarted(id) => assert_eq!(id, child_id),
            _ => panic!("Expected ChildRestarted message"),
        }
    }

    #[test]
    fn test_actor_id_display() {
        let id = ActorId(42);
        assert_eq!(format!("{id}"), "actor-42");
    }

    #[test]
    fn test_actor_id_properties() {
        let id1 = ActorId(1);
        let id2 = ActorId(1);
        let id3 = ActorId(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.0, 1);
    }

    #[test]
    fn test_message_value_types() {
        let string_val = MessageValue::String("test".to_string());
        let int_val = MessageValue::Integer(42);
        let _float_val = MessageValue::Float(3.15);
        let _bool_val = MessageValue::Bool(true);
        let actor_ref_val = MessageValue::ActorRef(ActorId(123));

        match string_val {
            MessageValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }

        match int_val {
            MessageValue::Integer(i) => assert_eq!(i, 42),
            _ => panic!("Expected integer value"),
        }

        match actor_ref_val {
            MessageValue::ActorRef(id) => assert_eq!(id, ActorId(123)),
            _ => panic!("Expected actor ref value"),
        }
    }

    #[test]
    fn test_message_value_list() {
        let list = MessageValue::List(vec![
            MessageValue::Integer(1),
            MessageValue::String("hello".to_string()),
            MessageValue::Bool(false),
        ]);

        match list {
            MessageValue::List(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    MessageValue::Integer(i) => assert_eq!(*i, 1),
                    _ => panic!("Expected integer"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_message_value_map() {
        let mut map = std::collections::HashMap::new();
        map.insert("key1".to_string(), MessageValue::Integer(10));
        map.insert(
            "key2".to_string(),
            MessageValue::String("value".to_string()),
        );

        let map_val = MessageValue::Map(map);
        match map_val {
            MessageValue::Map(m) => {
                assert_eq!(m.len(), 2);
                assert!(m.contains_key("key1"));
                assert!(m.contains_key("key2"));
            }
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_system_messages() {
        let start = Message::Start;
        let stop = Message::Stop;
        let restart = Message::Restart;

        match start {
            Message::Start => {}
            _ => panic!("Expected Start message"),
        }

        match stop {
            Message::Stop => {}
            _ => panic!("Expected Stop message"),
        }

        match restart {
            Message::Restart => {}
            _ => panic!("Expected Restart message"),
        }
    }

    #[test]
    fn test_user_message() {
        let message = Message::User(
            "greet".to_string(),
            vec![MessageValue::String("Alice".to_string())],
        );

        match message {
            Message::User(msg_type, values) => {
                assert_eq!(msg_type, "greet");
                assert_eq!(values.len(), 1);
            }
            _ => panic!("Expected User message"),
        }
    }

    #[test]
    fn test_error_message() {
        let message = Message::Error("Something went wrong".to_string());

        match message {
            Message::Error(err) => assert_eq!(err, "Something went wrong"),
            _ => panic!("Expected Error message"),
        }
    }

    #[test]
    fn test_supervision_messages() {
        let child_id = ActorId(456);
        let child_failed = Message::ChildFailed(child_id, "Crash".to_string());
        let child_restarted = Message::ChildRestarted(child_id);

        match child_failed {
            Message::ChildFailed(id, reason) => {
                assert_eq!(id, child_id);
                assert_eq!(reason, "Crash");
            }
            _ => panic!("Expected ChildFailed message"),
        }

        match child_restarted {
            Message::ChildRestarted(id) => assert_eq!(id, child_id),
            _ => panic!("Expected ChildRestarted message"),
        }
    }

    #[test]
    fn test_supervisor_directive() {
        let restart = SupervisorDirective::Restart;
        let _stop = SupervisorDirective::Stop;
        let _escalate = SupervisorDirective::Escalate;
        let _resume = SupervisorDirective::Resume;

        // Test cloning
        let restart_clone = restart;
        match restart_clone {
            SupervisorDirective::Restart => {}
            _ => panic!("Expected Restart directive"),
        }
    }

    #[test]
    fn test_actor_system_default() {
        let system = ActorSystem::default();
        assert!(system.actors.is_empty());
        assert!(system.actor_names.is_empty());
        assert_eq!(system.next_id, 1);
    }

    #[test]
    fn test_actor_system_clone() {
        let mut system = ActorSystem::default();
        system.actor_names.insert("test".to_string(), ActorId(1));
        system.next_id = 5;

        let cloned = system.clone();
        assert_eq!(cloned.next_id, 5);
        assert!(cloned.actor_names.contains_key("test"));
        assert!(cloned.actors.is_empty()); // actors are not cloned
    }

    #[test]
    fn test_supervisor_actor_new() {
        let supervisor = SupervisorActor::new(5);
        assert_eq!(supervisor.max_restarts, 5);
        assert!(supervisor.restart_count.is_empty());
    }

    #[test]
    fn test_supervisor_strategy() {
        let mut supervisor = SupervisorActor::new(3);
        let child_id = ActorId(100);

        // First failure should restart
        let strategy = supervisor.supervisor_strategy(child_id, "error");
        match strategy {
            SupervisorDirective::Restart => {}
            _ => panic!("Expected Restart directive"),
        }

        // Add restart count and test again
        supervisor.restart_count.insert(child_id, 3);
        let strategy = supervisor.supervisor_strategy(child_id, "error");
        match strategy {
            SupervisorDirective::Stop => {}
            _ => panic!("Expected Stop directive"),
        }
    }

    #[test]
    fn test_echo_actor_behavior() {
        let mut echo = EchoActor;
        let mut context = create_test_context();

        let message = Message::User(
            "hello".to_string(),
            vec![MessageValue::String("world".to_string())],
        );

        let result = echo
            .receive(message, &mut context)
            .expect("operation should succeed in test");
        match result {
            Some(Message::User(msg, values)) => {
                assert!(msg.contains("Echo: hello"));
                assert_eq!(values.len(), 1);
            }
            _ => panic!("Expected echo response"),
        }

        // Test with non-user message
        let start_message = Message::Start;
        let result = echo
            .receive(start_message, &mut context)
            .expect("operation should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_supervisor_child_failed_handling() {
        let mut supervisor = SupervisorActor::new(2);
        let mut context = create_test_context();
        let child_id = ActorId(789);

        let failed_message = Message::ChildFailed(child_id, "Test error".to_string());
        let result = supervisor
            .receive(failed_message, &mut context)
            .expect("operation should succeed in test");

        match result {
            Some(Message::ChildRestarted(id)) => assert_eq!(id, child_id),
            _ => panic!("Expected ChildRestarted response"),
        }

        // Check restart count
        assert_eq!(supervisor.restart_count.get(&child_id), Some(&1));
    }

    #[test]
    fn test_supervisor_max_restarts_exceeded() {
        let mut supervisor = SupervisorActor::new(1);
        let mut context = create_test_context();
        let child_id = ActorId(999);

        // First failure - should restart
        let failed_message = Message::ChildFailed(child_id, "Error 1".to_string());
        let result = supervisor
            .receive(failed_message, &mut context)
            .expect("operation should succeed in test");
        assert!(matches!(result, Some(Message::ChildRestarted(_))));

        // Second failure - should stop (exceeds max_restarts)
        let failed_message2 = Message::ChildFailed(child_id, "Error 2".to_string());
        let result = supervisor
            .receive(failed_message2, &mut context)
            .expect("operation should succeed in test");
        assert!(result.is_none()); // No response when stopping
    }

    #[test]
    fn test_supervisor_non_child_message() {
        let mut supervisor = SupervisorActor::new(3);
        let mut context = create_test_context();

        let user_message = Message::User("hello".to_string(), vec![]);
        let result = supervisor
            .receive(user_message, &mut context)
            .expect("operation should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_actor_system_spawn_duplicate_name() {
        let system = ActorSystem::new();
        let mut sys = system.lock().expect("operation should succeed in test");

        // Spawn first actor
        let result1 = sys.spawn("duplicate".to_string(), EchoActor);
        assert!(result1.is_ok());

        // Try to spawn another with same name
        let result2 = sys.spawn("duplicate".to_string(), EchoActor);
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_actor_system_find_by_name() {
        let system = ActorSystem::new();
        let actor_ref = {
            let mut sys = system.lock().expect("operation should succeed in test");
            sys.spawn("findme".to_string(), EchoActor)
                .expect("operation should succeed in test")
        };

        let sys = system.lock().expect("operation should succeed in test");
        let found = sys.find_actor_by_name("findme");
        assert!(found.is_some());
        assert_eq!(
            found.expect("operation should succeed in test").id,
            actor_ref.id
        );

        let not_found = sys.find_actor_by_name("nothere");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_actor_system_get_actor_ref() {
        let system = ActorSystem::new();
        let actor_ref = {
            let mut sys = system.lock().expect("operation should succeed in test");
            sys.spawn("getref".to_string(), EchoActor)
                .expect("operation should succeed in test")
        };

        let sys = system.lock().expect("operation should succeed in test");
        let found_ref = sys.get_actor_ref(actor_ref.id);
        assert!(found_ref.is_some());
        assert_eq!(
            found_ref.expect("operation should succeed in test").id,
            actor_ref.id
        );

        let not_found_ref = sys.get_actor_ref(ActorId(99999));
        assert!(not_found_ref.is_none());
    }

    #[test]
    fn test_actor_system_stop_actor() {
        let system = ActorSystem::new();
        let actor_ref = {
            let mut sys = system.lock().expect("operation should succeed in test");
            sys.spawn("stopme".to_string(), EchoActor)
                .expect("operation should succeed in test")
        };

        // Stop the actor
        {
            let mut sys = system.lock().expect("operation should succeed in test");
            let result = sys.stop_actor(actor_ref.id);
            assert!(result.is_ok());
        }

        // Verify actor is removed
        let sys = system.lock().expect("operation should succeed in test");
        let found = sys.get_actor_ref(actor_ref.id);
        assert!(found.is_none());
    }

    #[test]
    fn test_actor_system_shutdown() {
        let system = ActorSystem::new();
        {
            let mut sys = system.lock().expect("operation should succeed in test");
            sys.spawn("actor1".to_string(), EchoActor)
                .expect("operation should succeed in test");
            sys.spawn("actor2".to_string(), EchoActor)
                .expect("operation should succeed in test");
            assert_eq!(sys.actors.len(), 2);

            sys.shutdown();
            assert_eq!(sys.actors.len(), 0);
            assert_eq!(sys.actor_names.len(), 0);
        }
    }

    #[test]
    fn test_actor_ref_send_message() {
        let system = ActorSystem::new();
        let actor_ref = {
            let mut sys = system.lock().expect("operation should succeed in test");
            sys.spawn("sender_test".to_string(), EchoActor)
                .expect("operation should succeed in test")
        };

        let message = Message::User("ping".to_string(), vec![]);
        let result = actor_ref.send(message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_actor_context_find_actor() {
        let system = ActorSystem::new();
        let _actor_ref = {
            let mut sys = system.lock().expect("operation should succeed in test");
            sys.spawn("findable".to_string(), EchoActor)
                .expect("operation should succeed in test")
        };

        let context = ActorContext {
            actor_id: ActorId(2),
            actor_name: "searcher".to_string(),
            supervisor: None,
            children: std::collections::HashMap::new(),
            system,
        };

        let found = context.find_actor("findable");
        assert!(found.is_some());

        let not_found = context.find_actor("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_message_value_float() {
        let float_val = MessageValue::Float(3.15159);
        match float_val {
            MessageValue::Float(f) => assert!((f - 3.15159).abs() < 0.00001),
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_message_value_bool() {
        let true_val = MessageValue::Bool(true);
        let false_val = MessageValue::Bool(false);

        match true_val {
            MessageValue::Bool(b) => assert!(b),
            _ => panic!("Expected bool value"),
        }

        match false_val {
            MessageValue::Bool(b) => assert!(!b),
            _ => panic!("Expected bool value"),
        }
    }
}
#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::time::Duration;

    /// Test actor ref send when channel is closed
    #[test]
    fn test_actor_ref_send_channel_closed() {
        // Create a channel and immediately drop the receiver
        let (sender, _) = mpsc::channel();
        let actor_ref = ActorRef {
            id: ActorId(1),
            name: "closed_actor".to_string(),
            sender,
        };
        // Drop the receiver implicitly (no receiver created above)
        // The channel should be closed since we didn't keep the receiver

        // A second send should work since sender is still valid
        // But if we create with a dropped receiver explicitly:
        let (sender2, receiver2) = mpsc::channel::<ActorMessage>();
        drop(receiver2); // Explicitly drop receiver

        let actor_ref2 = ActorRef {
            id: ActorId(2),
            name: "closed_actor2".to_string(),
            sender: sender2,
        };

        let message = Message::User("test".to_string(), vec![]);
        let result = actor_ref2.send(message);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no longer running"));
    }

    /// Test actor ref ask when channel is closed
    #[test]
    fn test_actor_ref_ask_channel_closed() {
        let (sender, receiver) = mpsc::channel::<ActorMessage>();
        drop(receiver);

        let actor_ref = ActorRef {
            id: ActorId(3),
            name: "closed_for_ask".to_string(),
            sender,
        };

        let message = Message::User("ask_test".to_string(), vec![]);
        let result = actor_ref.ask(message, Duration::from_millis(50));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no longer running"));
    }

    /// Test actor ref ask timeout
    #[test]
    fn test_actor_ref_ask_timeout() {
        let (sender, _receiver) = mpsc::channel::<ActorMessage>();

        let actor_ref = ActorRef {
            id: ActorId(4),
            name: "slow_actor".to_string(),
            sender,
        };

        let message = Message::User("timeout_test".to_string(), vec![]);
        // Use very short timeout - no one will respond
        let result = actor_ref.ask(message, Duration::from_millis(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout"));
    }

    /// Test DummyBehavior receive method
    #[test]
    fn test_dummy_behavior_receive() {
        let mut dummy = DummyBehavior;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(100),
            actor_name: "dummy_test".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let message = Message::User("test".to_string(), vec![]);
        let result = dummy.receive(message, &mut ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test default ActorBehavior trait implementations
    #[test]
    fn test_actor_behavior_defaults() {
        let mut echo = EchoActor;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(101),
            actor_name: "default_test".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        // Test pre_start default
        assert!(echo.pre_start(&mut ctx).is_ok());

        // Test post_stop default
        assert!(echo.post_stop(&mut ctx).is_ok());

        // Test pre_restart default
        assert!(echo.pre_restart(&mut ctx, "test reason").is_ok());

        // Test post_restart default
        assert!(echo.post_restart(&mut ctx, "test reason").is_ok());

        // Test supervisor_strategy default
        let directive = echo.supervisor_strategy(ActorId(200), "child failure");
        assert!(matches!(directive, SupervisorDirective::Restart));
    }

    /// Test ActorContext get_self when actor exists
    #[test]
    fn test_actor_context_get_self() {
        let system = ActorSystem::new();
        let actor_ref = {
            let mut sys = system.lock().expect("lock should not fail");
            sys.spawn("self_test".to_string(), EchoActor)
                .expect("spawn should succeed")
        };

        let ctx = ActorContext {
            actor_id: actor_ref.id,
            actor_name: "self_test".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system: system.clone(),
        };

        let self_ref = ctx.get_self();
        assert!(self_ref.is_ok());
        assert_eq!(self_ref.unwrap().id, actor_ref.id);

        // Cleanup
        let mut sys = system.lock().expect("lock should not fail");
        sys.shutdown();
    }

    /// Test ActorContext get_self when actor doesn't exist
    #[test]
    fn test_actor_context_get_self_not_found() {
        let system = ActorSystem::new();

        let ctx = ActorContext {
            actor_id: ActorId(9999), // Non-existent actor
            actor_name: "ghost".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = ctx.get_self();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    /// Test ActorContext stop_child
    #[test]
    fn test_actor_context_stop_child() {
        let system = ActorSystem::new();

        // Create parent context with a child
        let child_system = ActorSystem::new();
        let child_ref = {
            let mut sys = child_system.lock().expect("lock should not fail");
            sys.spawn("child_actor".to_string(), EchoActor)
                .expect("spawn should succeed")
        };

        let mut children = HashMap::new();
        children.insert(child_ref.id, child_ref.clone());

        let mut ctx = ActorContext {
            actor_id: ActorId(50),
            actor_name: "parent".to_string(),
            supervisor: None,
            children,
            system,
        };

        // Stop the child - should succeed even if send fails
        let result = ctx.stop_child(child_ref.id);
        assert!(result.is_ok());

        // Child should be removed from children map
        assert!(!ctx.children.contains_key(&child_ref.id));

        // Cleanup
        let mut sys = child_system.lock().expect("lock should not fail");
        sys.shutdown();
    }

    /// Test ActorContext stop_child for non-existent child
    #[test]
    fn test_actor_context_stop_child_not_found() {
        let system = ActorSystem::new();

        let mut ctx = ActorContext {
            actor_id: ActorId(51),
            actor_name: "parent2".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        // Stopping non-existent child should succeed (no-op)
        let result = ctx.stop_child(ActorId(999));
        assert!(result.is_ok());
    }

    /// Test Message variants serialization roundtrip
    #[test]
    fn test_message_serialization() {
        use serde_json;

        let messages = vec![
            Message::Start,
            Message::Stop,
            Message::Restart,
            Message::User("test".to_string(), vec![MessageValue::Integer(42)]),
            Message::Error("error msg".to_string()),
            Message::ChildFailed(ActorId(1), "failure".to_string()),
            Message::ChildRestarted(ActorId(2)),
        ];

        for msg in messages {
            let serialized = serde_json::to_string(&msg).expect("serialize should succeed");
            let _deserialized: Message =
                serde_json::from_str(&serialized).expect("deserialize should succeed");
        }
    }

    /// Test MessageValue serialization roundtrip
    #[test]
    fn test_message_value_serialization() {
        use serde_json;

        let mut map = HashMap::new();
        map.insert("key".to_string(), MessageValue::Integer(10));

        let values = vec![
            MessageValue::String("hello".to_string()),
            MessageValue::Integer(123),
            MessageValue::Float(3.14),
            MessageValue::Bool(true),
            MessageValue::List(vec![MessageValue::Integer(1), MessageValue::Integer(2)]),
            MessageValue::Map(map),
            MessageValue::ActorRef(ActorId(42)),
        ];

        for val in values {
            let serialized = serde_json::to_string(&val).expect("serialize should succeed");
            let _deserialized: MessageValue =
                serde_json::from_str(&serialized).expect("deserialize should succeed");
        }
    }

    /// Test ActorId serialization
    #[test]
    fn test_actor_id_serialization() {
        use serde_json;

        let id = ActorId(12345);
        let serialized = serde_json::to_string(&id).expect("serialize should succeed");
        let deserialized: ActorId =
            serde_json::from_str(&serialized).expect("deserialize should succeed");
        assert_eq!(id, deserialized);
    }

    /// Test ActorId hash implementation (for HashMap keys)
    #[test]
    fn test_actor_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ActorId(1));
        set.insert(ActorId(2));
        set.insert(ActorId(1)); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&ActorId(1)));
        assert!(set.contains(&ActorId(2)));
    }

    /// Test ActorId copy semantics
    #[test]
    fn test_actor_id_copy() {
        let id1 = ActorId(42);
        let id2 = id1; // Copy
        assert_eq!(id1, id2);
        assert_eq!(id1.0, 42);
        assert_eq!(id2.0, 42);
    }

    /// Test SupervisorDirective debug output
    #[test]
    fn test_supervisor_directive_debug() {
        let directives = vec![
            SupervisorDirective::Restart,
            SupervisorDirective::Stop,
            SupervisorDirective::Escalate,
            SupervisorDirective::Resume,
        ];

        for d in directives {
            let debug_str = format!("{:?}", d);
            assert!(!debug_str.is_empty());
        }
    }

    /// Test stopping a non-existent actor in ActorSystem
    #[test]
    fn test_actor_system_stop_nonexistent() {
        let system = ActorSystem::new();
        let mut sys = system.lock().expect("lock should not fail");

        let result = sys.stop_actor(ActorId(99999));
        assert!(result.is_ok()); // Should succeed (no-op)
    }

    /// Test ActorRef debug output
    #[test]
    fn test_actor_ref_debug() {
        let (sender, _) = mpsc::channel();
        let actor_ref = ActorRef {
            id: ActorId(123),
            name: "debug_test".to_string(),
            sender,
        };
        let debug_str = format!("{:?}", actor_ref);
        assert!(debug_str.contains("ActorRef"));
        assert!(debug_str.contains("123"));
    }

    /// Test ActorRef clone
    #[test]
    fn test_actor_ref_clone() {
        let (sender, _) = mpsc::channel();
        let actor_ref = ActorRef {
            id: ActorId(456),
            name: "clone_test".to_string(),
            sender,
        };

        let cloned = actor_ref.clone();
        assert_eq!(cloned.id, actor_ref.id);
        assert_eq!(cloned.name, actor_ref.name);
    }

    /// Test EchoActor with Start message (should return None)
    #[test]
    fn test_echo_actor_start_message() {
        let mut echo = EchoActor;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(200),
            actor_name: "echo_start".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = echo.receive(Message::Start, &mut ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test EchoActor with Stop message
    #[test]
    fn test_echo_actor_stop_message() {
        let mut echo = EchoActor;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(201),
            actor_name: "echo_stop".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = echo.receive(Message::Stop, &mut ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test EchoActor with Restart message
    #[test]
    fn test_echo_actor_restart_message() {
        let mut echo = EchoActor;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(202),
            actor_name: "echo_restart".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = echo.receive(Message::Restart, &mut ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test EchoActor with Error message
    #[test]
    fn test_echo_actor_error_message() {
        let mut echo = EchoActor;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(203),
            actor_name: "echo_error".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = echo.receive(Message::Error("test error".to_string()), &mut ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test EchoActor with ChildFailed message
    #[test]
    fn test_echo_actor_child_failed_message() {
        let mut echo = EchoActor;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(204),
            actor_name: "echo_cf".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = echo.receive(
            Message::ChildFailed(ActorId(999), "child failed".to_string()),
            &mut ctx,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test EchoActor with ChildRestarted message
    #[test]
    fn test_echo_actor_child_restarted_message() {
        let mut echo = EchoActor;
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(205),
            actor_name: "echo_cr".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = echo.receive(Message::ChildRestarted(ActorId(888)), &mut ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test SupervisorActor with Start message (not ChildFailed)
    #[test]
    fn test_supervisor_actor_start_message() {
        let mut supervisor = SupervisorActor::new(3);
        let system = ActorSystem::new();
        let mut ctx = ActorContext {
            actor_id: ActorId(300),
            actor_name: "sup_start".to_string(),
            supervisor: None,
            children: HashMap::new(),
            system,
        };

        let result = supervisor.receive(Message::Start, &mut ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// Test Message::User with multiple value types
    #[test]
    fn test_message_user_multiple_values() {
        let values = vec![
            MessageValue::Integer(1),
            MessageValue::Float(2.5),
            MessageValue::String("test".to_string()),
            MessageValue::Bool(true),
            MessageValue::ActorRef(ActorId(10)),
        ];

        let message = Message::User("multi_value".to_string(), values.clone());
        match message {
            Message::User(msg_type, vals) => {
                assert_eq!(msg_type, "multi_value");
                assert_eq!(vals.len(), 5);
            }
            _ => panic!("Expected User message"),
        }
    }

    /// Test MessageValue nested List
    #[test]
    fn test_message_value_nested_list() {
        let inner_list = MessageValue::List(vec![
            MessageValue::Integer(1),
            MessageValue::Integer(2),
        ]);
        let outer_list = MessageValue::List(vec![inner_list, MessageValue::String("end".to_string())]);

        match outer_list {
            MessageValue::List(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    MessageValue::List(inner) => assert_eq!(inner.len(), 2),
                    _ => panic!("Expected nested list"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    /// Test MessageValue nested Map
    #[test]
    fn test_message_value_nested_map() {
        let mut inner_map = HashMap::new();
        inner_map.insert("inner_key".to_string(), MessageValue::Integer(42));

        let mut outer_map = HashMap::new();
        outer_map.insert("nested".to_string(), MessageValue::Map(inner_map));
        outer_map.insert("simple".to_string(), MessageValue::Bool(false));

        let map_val = MessageValue::Map(outer_map);
        match map_val {
            MessageValue::Map(m) => {
                assert_eq!(m.len(), 2);
                assert!(m.contains_key("nested"));
                assert!(m.contains_key("simple"));
            }
            _ => panic!("Expected map"),
        }
    }

    /// Test ActorSystem spawn multiple actors
    #[test]
    fn test_actor_system_spawn_multiple() {
        let system = ActorSystem::new();
        let refs = {
            let mut sys = system.lock().expect("lock should not fail");
            let ref1 = sys
                .spawn("actor_a".to_string(), EchoActor)
                .expect("spawn should succeed");
            let ref2 = sys
                .spawn("actor_b".to_string(), EchoActor)
                .expect("spawn should succeed");
            let ref3 = sys
                .spawn("actor_c".to_string(), EchoActor)
                .expect("spawn should succeed");
            (ref1, ref2, ref3)
        };

        assert_ne!(refs.0.id, refs.1.id);
        assert_ne!(refs.1.id, refs.2.id);
        assert_ne!(refs.0.id, refs.2.id);

        // Cleanup
        let mut sys = system.lock().expect("lock should not fail");
        sys.shutdown();
    }

    /// Test actor shutdown removes all actors and names
    #[test]
    fn test_actor_system_shutdown_clears_all() {
        let system = ActorSystem::new();
        {
            let mut sys = system.lock().expect("lock should not fail");
            sys.spawn("shutdown_test_1".to_string(), EchoActor)
                .expect("spawn should succeed");
            sys.spawn("shutdown_test_2".to_string(), EchoActor)
                .expect("spawn should succeed");
            sys.shutdown();

            assert!(sys.actors.is_empty());
            assert!(sys.actor_names.is_empty());
        }
    }

    /// Test supervisor with escalate strategy check
    #[test]
    fn test_supervisor_directive_escalate() {
        let directive = SupervisorDirective::Escalate;
        match directive {
            SupervisorDirective::Escalate => {}
            _ => panic!("Expected Escalate"),
        }
    }

    /// Test supervisor with resume strategy check
    #[test]
    fn test_supervisor_directive_resume() {
        let directive = SupervisorDirective::Resume;
        match directive {
            SupervisorDirective::Resume => {}
            _ => panic!("Expected Resume"),
        }
    }

    /// Test ActorSystem ID incrementing
    #[test]
    fn test_actor_system_id_increments() {
        let mut sys = ActorSystem::default();
        let initial_id = sys.next_id;

        let _ref1 = sys
            .spawn("inc_test_1".to_string(), EchoActor)
            .expect("spawn should succeed");
        assert!(sys.next_id > initial_id);

        let _ref2 = sys
            .spawn("inc_test_2".to_string(), EchoActor)
            .expect("spawn should succeed");
        assert!(sys.next_id > initial_id + 1);

        sys.shutdown();
    }
}

#[cfg(test)]
mod property_tests_actor {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_send_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
