// Look at this absolute mess. What have you done?

#[macro_export]
macro_rules! duck_impl {
    // auto &self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?;
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged() {
                TaggedSsoString64::Short(short) => short.$method($($($value),*)?),
                TaggedSsoString64::Long(long) => long.$method($($($value),*)?),
            }
        }
    };

    // auto unsafe &self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?;
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged() {
                TaggedSsoString64::Short(short) => short.$method($($($value),*)?),
                TaggedSsoString64::Long(long) => long.$method($($($value),*)?),
            }
        }
    };

    // &self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self as $this:ident
            $(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?
        $body:block
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged_mut() {
                TaggedSsoString64Mut::Short($this) => $body,
                TaggedSsoString64Mut::Long($this) => $body,
            }
        }
    };

    // &unsafe self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self as $this:ident
            $(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?
        $body:block
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged_mut() {
                TaggedSsoString64Mut::Short($this) => $body,
                TaggedSsoString64Mut::Long($this) => $body,
            }
        }
    };

    // auto &mut self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?;
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged_mut() {
                TaggedSsoString64Mut::Short(short) => short.$method($($($value),*)?),
                TaggedSsoString64Mut::Long(long) => long.$method($($($value),*)?),
            }
        }
    };

    // auto unsafe &mut self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?;
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged_mut() {
                TaggedSsoString64Mut::Short(short) => short.$method($($($value),*)?),
                TaggedSsoString64Mut::Long(long) => long.$method($($($value),*)?),
            }
        }
    };

    // &mut self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self as $this:ident
            $(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?
        $body:block
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged_mut() {
                TaggedSsoString64Mut::Short($this) => $body,
                TaggedSsoString64Mut::Long($this) => $body,
            }
        }
    };

    // unsafe &mut self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self as $this:ident
            $(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?
        $body:block
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            match self.tagged_mut() {
                TaggedSsoString64Mut::Short($this) => $body,
                TaggedSsoString64Mut::Long($this) => $body,
            }
        }
    };
}

pub const TODO_IMPL_MESSAGE: &'static str =
    "This method exists on std::string::String, but does not yet exist on SsoString";

#[macro_export]
macro_rules! todo_impl {
    // auto
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            $($value:ident: $T:ty),*$(,)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?($($value: $T),*) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };

    // auto unsafe 
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            $($value:ident: $T:ty),*$(,)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?($($value: $T),*) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };

    // auto self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };

    // auto unsafe self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };

    // auto &self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };

    // auto unsafe &self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };

    // auto &mut self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };

    // auto unsafe &mut self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::TODO_IMPL_MESSAGE)
        }
    };
}

pub const NEVER_IMPL_MESSAGE: &'static str = concat!(
    "This method exists on std::string::String, but will never exist on SsoString because of ",
    "trade-offs required to allow for the optimisation."
);

#[macro_export]
macro_rules! never_impl {
    // auto &self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::NEVER_IMPL_MESSAGE)
        }
    };

    // auto unsafe &self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::NEVER_IMPL_MESSAGE)
        }
    };

    // auto &mut self
    {
        $(#[$attr:meta])*
        $vis:vis fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis fn $method$(<
            $($($a),+,)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::NEVER_IMPL_MESSAGE)
        }
    };

    // auto unsafe &mut self
    {
        $(#[$attr:meta])*
        $vis:vis unsafe fn $method:ident
        $(<
            $($($a:lifetime),+$(,)?)?
            $($TParam:ident),*$(,)?
        >)?(
            &mut self$(,$($value:ident: $T:ty),*$(,)?)?
        ) $(-> $Returns:ty)? $(where $($wherett:tt)*)?$(;)?
    } => {
        $(#[$attr])*
        $vis unsafe fn $method$(<
            $($($a:lifetime),+$(,)?)?
            $($TParam),*
        >)?(
            &mut self, $($($value: $T),*)?
        ) $(-> $Returns)? 
        $(where $($wherett)*)?
        {
            todo!("{}", $crate::impl_macros::NEVER_IMPL_MESSAGE)
        }
    };
}
