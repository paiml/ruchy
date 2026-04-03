# Sub-spec: System Operations — Core Runtime and System APIs

**Parent:** [systems-operations-io-spec.md](../systems-operations-io-spec.md) Sections 1-3

---

### 1.1 Command Line Interface

**Current Status**: ✅ Partially Implemented  
**Gap**: Missing subcommands and options

```ruchy
// Required CLI commands (like Deno)
ruchy run <script.ruchy> [args...]        // ✅ Implemented
ruchy check <script.ruchy>                // ✅ Implemented  
ruchy test [pattern]                      // ✅ Implemented (needs work)
ruchy fmt [files...]                      // ✅ Implemented (outputs AST)
ruchy lint [files...]                     // ✅ Implemented
ruchy compile <script> -o <binary>        // ❌ Not Implemented
ruchy repl                                // ✅ Implemented
ruchy doc <files...>                      // ✅ Implemented
ruchy bench <files...>                    // ✅ Implemented
ruchy upgrade                             // ❌ Not Implemented
ruchy install <package>                   // ❌ Not Implemented
```

### 1.2 Runtime Permissions Model

**Current Status**: ❌ Not Implemented  
**Required**: Fine-grained permission system

```ruchy
// Permission flags (like Deno)
ruchy run --allow-read=/path script.ruchy
ruchy run --allow-write=/path script.ruchy
ruchy run --allow-net=domain.com script.ruchy
ruchy run --allow-env=VAR1,VAR2 script.ruchy
ruchy run --allow-run=git,make script.ruchy
ruchy run --allow-all script.ruchy  // Development mode
```

## 2. System Operations API

### 2.1 Process Management

**Current Status**: ❌ Not Implemented  
**Priority**: CRITICAL

```ruchy
// Process spawning and control
mod std::process {
    struct Command {
        fn new(program: String) -> Command
        fn arg(self, arg: String) -> Command
        fn args(self, args: [String]) -> Command
        fn env(self, key: String, value: String) -> Command
        fn current_dir(self, dir: String) -> Command
        fn stdin(self, cfg: Stdio) -> Command
        fn stdout(self, cfg: Stdio) -> Command
        fn stderr(self, cfg: Stdio) -> Command
        fn spawn(self) -> Result<Child, Error>
        fn output(self) -> Result<Output, Error>
        fn status(self) -> Result<ExitStatus, Error>
    }
    
    struct Child {
        fn id() -> u32
        fn kill() -> Result<(), Error>
        fn wait() -> Result<ExitStatus, Error>
        fn wait_with_output() -> Result<Output, Error>
        stdin: Option<ChildStdin>
        stdout: Option<ChildStdout>
        stderr: Option<ChildStderr>
    }
    
    struct Output {
        status: ExitStatus
        stdout: Bytes
        stderr: Bytes
    }
    
    enum Stdio {
        Inherit,
        Piped,
        Null,
    }
}

// Example usage
let result = Command::new("apt-get")
    .args(["install", "-y", "curl"])
    .stdout(Stdio::Piped)
    .output()?
```

### 2.2 File System Operations

**Current Status**: ❌ Not Implemented  
**Priority**: CRITICAL

```ruchy
mod std::fs {
    // File reading
    fn read_to_string(path: String) -> Result<String, Error>
    fn read(path: String) -> Result<Bytes, Error>
    
    // File writing
    fn write(path: String, contents: Bytes) -> Result<(), Error>
    fn write_string(path: String, contents: String) -> Result<(), Error>
    fn append(path: String, contents: Bytes) -> Result<(), Error>
    
    // File metadata
    fn metadata(path: String) -> Result<Metadata, Error>
    fn exists(path: String) -> bool
    fn is_file(path: String) -> bool
    fn is_dir(path: String) -> bool
    
    // Directory operations
    fn create_dir(path: String) -> Result<(), Error>
    fn create_dir_all(path: String) -> Result<(), Error>
    fn remove_dir(path: String) -> Result<(), Error>
    fn remove_dir_all(path: String) -> Result<(), Error>
    fn read_dir(path: String) -> Result<DirIterator, Error>
    
    // File operations
    fn copy(from: String, to: String) -> Result<u64, Error>
    fn rename(from: String, to: String) -> Result<(), Error>
    fn remove_file(path: String) -> Result<(), Error>
    
    // Permissions
    fn set_permissions(path: String, perm: Permissions) -> Result<(), Error>
    
    // Temporary files
    fn temp_dir() -> String
    fn create_temp_dir(prefix: String) -> Result<TempDir, Error>
    
    struct Metadata {
        fn len() -> u64
        fn is_file() -> bool
        fn is_dir() -> bool
        fn is_symlink() -> bool
        fn modified() -> Result<SystemTime, Error>
        fn accessed() -> Result<SystemTime, Error>
        fn created() -> Result<SystemTime, Error>
        fn permissions() -> Permissions
    }
    
    struct Permissions {
        fn readonly() -> bool
        fn set_readonly(readonly: bool)
        fn mode() -> u32  // Unix only
        fn set_mode(mode: u32)  // Unix only
    }
}
```

### 2.3 Environment Variables

**Current Status**: ❌ Not Implemented  
**Priority**: HIGH

```ruchy
mod std::env {
    fn var(key: String) -> Result<String, Error>
    fn var_os(key: String) -> Option<OsString>
    fn set_var(key: String, value: String)
    fn remove_var(key: String)
    fn vars() -> Map<String, String>
    fn current_dir() -> Result<PathBuf, Error>
    fn set_current_dir(path: Path) -> Result<(), Error>
    fn home_dir() -> Option<PathBuf>
    fn temp_dir() -> PathBuf
    fn current_exe() -> Result<PathBuf, Error>
    fn args() -> [String]
}
```

### 2.4 User and Permissions

**Current Status**: ❌ Not Implemented  
**Priority**: HIGH

```ruchy
mod std::os::unix {
    fn uid() -> u32
    fn gid() -> u32
    fn effective_uid() -> u32
    fn effective_gid() -> u32
    fn username() -> Option<String>
    fn hostname() -> Result<String, Error>
}
```

## 3. I/O Operations

### 3.1 Standard I/O

**Current Status**: ⚠️ Partially Implemented (println only)  
**Priority**: CRITICAL

```ruchy
mod std::io {
    // Standard streams
    fn stdin() -> Stdin
    fn stdout() -> Stdout
    fn stderr() -> Stderr
    
    struct Stdin {
        fn read_line() -> Result<String, Error>
        fn read_all() -> Result<String, Error>
        fn read_bytes(n: usize) -> Result<Bytes, Error>
        fn is_terminal() -> bool
    }
    
    struct Stdout {
        fn write(data: Bytes) -> Result<usize, Error>
        fn write_all(data: Bytes) -> Result<(), Error>
        fn flush() -> Result<(), Error>
        fn is_terminal() -> bool
    }
    
    struct Stderr {
        fn write(data: Bytes) -> Result<usize, Error>
        fn write_all(data: Bytes) -> Result<(), Error>
        fn flush() -> Result<(), Error>
        fn is_terminal() -> bool
    }
    
    // Print functions (currently only println exists)
    fn print(msg: String)           // ❌ Not Implemented
    fn println(msg: String)          // ✅ Implemented
    fn eprint(msg: String)           // ❌ Not Implemented
    fn eprintln(msg: String)         // ❌ Not Implemented
    
    // Formatted printing
    fn format(fmt: String, args: ...) -> String  // ❌ Not Implemented
}
```

### 3.2 File I/O

**Current Status**: ❌ Not Implemented  
**Priority**: CRITICAL

```ruchy
mod std::fs {
    struct File {
        fn open(path: String) -> Result<File, Error>
        fn create(path: String) -> Result<File, Error>
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error>
        fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error>
        fn sync_all(&self) -> Result<(), Error>
        fn sync_data(&self) -> Result<(), Error>
        fn set_len(&self, size: u64) -> Result<(), Error>
        fn metadata(&self) -> Result<Metadata, Error>
    }
    
    enum SeekFrom {
        Start(u64),
        End(i64),
        Current(i64),
    }
}
```

### 3.3 Network I/O

**Current Status**: ❌ Not Implemented  
**Priority**: MEDIUM (needed for package management)

```ruchy
mod std::net {
    // HTTP Client
    fn fetch(url: String, options: FetchOptions) -> Result<Response, Error>
    
    struct FetchOptions {
        method: String,
        headers: Map<String, String>,
        body: Option<Bytes>,
        timeout: Option<Duration>,
    }
    
    struct Response {
        status: u16,
        headers: Map<String, String>,
        fn text() -> Result<String, Error>
        fn json<T>() -> Result<T, Error>
        fn bytes() -> Result<Bytes, Error>
    }
    
    // TCP
    struct TcpListener {
        fn bind(addr: String) -> Result<TcpListener, Error>
        fn accept() -> Result<(TcpStream, SocketAddr), Error>
    }
    
    struct TcpStream {
        fn connect(addr: String) -> Result<TcpStream, Error>
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error>
    }
}
```
