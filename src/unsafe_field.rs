/// Indicates that a field is unsafe to write to, since we have to uphold certain invariants.
/// Make sure to document them! 
/// 
/// # Safety
/// - **Declaring** this struct is unsafe.
/// - within a struct, all fields should have a different associated constant, I would suggest just
///   numbering them from 0.
/// 
/// ```rs
/// type Boolean = u8;
/// let mut valid_bool = unsafe { UnsafeWrite::new(true as Boolean) };
/// let mut always_3 = unsafe { UnsafeWrite::new(3) };
/// // UB
/// mem::swap(valid_bool, always_3);
/// ```
/// 
/// # Theoretical Best Implementation
/// 
/// A better implementation of this is not possible without a macro. I would consider a macro 
/// implementation a good enough proof for item-only scoping being possible all the time. This is 
/// perhaps a bad name, but item-only scoping, means that we can write (all?) unsafe code to be 
/// verifiably sound at item-scope, this includes
/// 
/// - Struct declaration
/// - Struct construction
/// - Function declaration
/// - Function calling
/// 
/// For example, in the case of `LongString`, the following declaration is unsound. This is is 
/// because the type `usize` does not follow the same contract as the field `len`, therefore it is 
/// and unsafe type to use for `len`. As a result, we must mark it as such. The same is the case 
/// for all other fields, since they all have invariants.
/// 
/// ```rs
/// /// # Safety
/// /// - `len` must be constrained by [len invariants] at all times
/// /// - `capacity` must be contrained by [capacity invariants] at all times
/// /// - `buf` must constrained by [buf invaraiants] at all times
/// struct LongString {
///     len: usize,
///     capacity: usize,
///     buf: RawBuf<usize>,
/// }
/// ```
/// 
/// Our declaration should soundly look like this:
/// 
/// ```rs
/// struct LongString {
///     unsafe len: usize,
///     unsafe capacity: usize,
///     unsafe buf: RawBuf<usize>,
/// }
/// ```
/// 
/// More formally, the encapsulation of struct-declaration unsafe scoping is as follows:
/// - A safety contract must be written above the struct declaration
/// - The safety contract must include **only** the invariants for each type. These invariants 
///   should ideally be upheld *at all times*. I would suggest *always*, but this might not allow 
///   for all possible data structures... I have a hunch it is though.
/// - If a field has an invariant that is already encapsulated safely by the type it is assigned,
///   we do not need to write about that invariant (though you can if you want??)
/// - If a field has an invariant that is not encapsulated safely by the type it is assigned, you
///   MUST declare it as `unsafe`.
/// 
/// It is potentially not obvious why this is encapsulated at the item level. Consider 
/// function-execution unsafe scoping. The function defines a contract, and if we fulfill that 
/// contract, we can execute the function *completely safely*. Generally, 
/// 
/// 1. Define a contract for an unsafe item, such that
/// 2. if we validate that contract
/// 3. we can use the item safely 
/// 
/// With struct-declaration unsafe scoping, we are doing essentially the same thing:
/// 
/// 1. Define a contract (for an unsafe item?) such that
/// 2. if we validate that contract (define the struct properly)
/// 3. we can use the item safely (define methods on the struct etc.)
/// 
/// The bit "for an unsafe item" might cause some disagreement. Who's to say that something is an 
/// unsafe item? Well, in this world all struct declarations are unsafe, except for those without 
/// a safety contract... Actually, that's the same as all functions in this world. All functions 
/// are unsafe, except for those without a safety contract. 
#[derive(Clone, Copy)]
pub struct UnsafeWrite<T, const FIELD_INDEX: usize>(T);

impl<T, const FIELD_INDEX: usize> UnsafeWrite<T, FIELD_INDEX> {
    /// Constructs a new [`UnsafeWrite`]
    ///
    /// # Safety
    /// - must uphold all invariants of the field
    pub const unsafe fn new(value: T) -> Self {
        Self(value)
    }

    /// Sets the underyling value to `value`
    ///
    /// # Safety
    /// - must uphold all invariants of the field
    pub unsafe fn set(&mut self, value: T) {
        self.0 = value;
    }

    /// Return a reference to the underlying value
    pub const fn get(&self) -> &T {
        &self.0
    }

    pub fn own(self) -> T {
        self.0
    }
}