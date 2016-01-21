use std::fmt;
use std::error::Error;


pub enum SpawnError<S: Sized> {
    /// The State Machine Slab capacity is reached
    ///
    /// The capacity is configured in the `rotor::Config` and is used
    /// for creating `rotor::Loop`.
    ///
    /// The item in this struct is the Seed that send to create a machine
    NoSlabSpace(S),
    /// Error returned from `Machine::create` handler
    UserError(Box<Error>),
}

impl<S> fmt::Display for SpawnError<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::SpawnError::*;
        match *self {
            NoSlabSpace(_) => {
                write!(fmt, "state machine slab capacity limit is reached")
            }
            UserError(ref err) => {
                write!(fmt, "{}", err)
            }
        }
    }
}

impl<S> SpawnError<S> {
    pub fn description(&self) -> &str {
        use self::SpawnError::*;
        match self {
            &NoSlabSpace(_) => "state machine slab capacity limit is reached",
            &UserError(ref err) => err.description(),
        }
    }
    pub fn cause(&self) -> Option<&Error> {
        use self::SpawnError::*;
        match self {
            &NoSlabSpace(_) => None,
            &UserError(ref err) => Some(&**err),
        }
    }
    pub fn map<T:Sized, F: FnOnce(S) -> T>(self, fun:F) -> SpawnError<T> {
        use self::SpawnError::*;
        match self {
            NoSlabSpace(x) => NoSlabSpace(fun(x)),
            UserError(e) => UserError(e),
        }
    }
}
impl<S: Error> Error for SpawnError<S> {
    fn description(&self) -> &str {
        self.description()
    }
    fn cause(&self) -> Option<&Error> {
        self.cause()
    }
}

impl<S> From<Box<Error>> for SpawnError<S> {
    fn from(x: Box<Error>) -> SpawnError<S> {
        SpawnError::UserError(x)
    }
}

impl<S> fmt::Debug for SpawnError<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::SpawnError::*;
        match *self {
            NoSlabSpace(..) => {
                write!(fmt, "NoSlabSpace(<hidden seed>)")
            }
            UserError(ref err) => {
                write!(fmt, "UserError({:?})", err)
            }
        }
    }
}
