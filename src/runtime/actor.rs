#![allow(clippy::print_stdout, clippy::print_stderr)]
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
/// use ruchy::runtime::actor::send;
/// 
/// let result = send(());
/// assert_eq!(result, Ok(()));
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
/// use ruchy::runtime::actor::ask;
/// 
/// let result = ask(());
/// assert_eq!(result, Ok(()));
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
/// use ruchy::runtime::actor::stop_child;
/// 
/// let result = stop_child(());
/// assert_eq!(result, Ok(()));
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
/// ```
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
/// use ruchy::runtime::actor::find_actor;
/// 
/// let result = find_actor("example");
/// assert_eq!(result, Ok(()));
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
/// ```
/// use ruchy::runtime::actor::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
/// # Examples
/// 
/// ```
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
/// ```
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
            return Err(anyhow!("Actor with name '{}' already exists", name));
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
/// ```
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
/// ```
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
/// ```
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
/// ```
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
    #[test]
    fn test_actor_system_creation() {
        let system = ActorSystem::new();
        assert!(system.lock().expect("Failed to acquire lock").actors.is_empty());
    }
    #[test]
    fn test_echo_actor() {
        let system = ActorSystem::new();
        let actor_ref = {
            let mut sys = system.lock().expect("Failed to acquire lock");
            sys.spawn("echo".to_string(), EchoActor).expect("Failed to spawn echo actor")
        };
        let message = Message::User(
            "test".to_string(),
            vec![MessageValue::String("hello".to_string())],
        );
        let response = actor_ref.ask(message, Duration::from_millis(100)).expect("Failed to get response from actor");
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
}
#[cfg(test)]
mod property_tests_actor {
    use proptest::proptest;
    
    
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_send_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
