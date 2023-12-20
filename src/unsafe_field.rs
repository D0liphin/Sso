use std::ptr::NonNull;

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
/// let mut valid_bool = unsafe { UnsafeField::new(true as Boolean) };
/// let mut always_3 = unsafe { UnsafeField::new(3) };
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
pub struct UnsafeField<T, const FIELD_INDEX: usize>(T);

impl<T, const FIELD_INDEX: usize> UnsafeField<T, FIELD_INDEX> {
    /// Return a reference to the underlying value
    pub const fn get(&self) -> &T {
        &self.0
    }

    pub fn own(self) -> T {
        self.0
    }
}

impl<T, const FIELD_INDEX: usize> UnsafeAssign<T> for UnsafeField<T, FIELD_INDEX> {
    unsafe fn new(value: T) -> Self {
        Self(value)
    }

    unsafe fn set(&mut self, value: T) {
        self.0 = value;
    }

    fn get_mut(&mut self) -> NonNull<T> {
        NonNull::from(&mut self.0)
    }
}

pub trait UnsafeAssign<T>
where
    Self: Sized,
{
    /// Constructs a new [`UnsafeField`]
    ///
    /// # Safety
    /// - must uphold all invariants of the field
    unsafe fn new(value: T) -> Self;

    /// Sets the underyling value to `value`
    ///
    /// # Safety
    /// - must uphold all invariants of the field
    unsafe fn set(&mut self, value: T);

    /// Gets a raw pointer to the value
    /// 
    /// # Safety
    /// - msut uphold all invaraints when assigning the pointer
    fn get_mut(&mut self) -> NonNull<T>;
}

/// Assign several [`UnsafeField`] 'simultaneously'.
///
/// There might be occasions where we cannot assign multiple fields simultaenously by
/// reconstructing the struct (though this should be done in most cases). In this case, we
/// can enforce a slightly lesser form of safety, by upholding invariants "only when the struct"
/// is read from. This pattern guarantees that we cannot get a `&self` in between writes to
/// fields.
///
/// ```rs
/// unsafe_field::SimultaneousUnsafeAssignment
///     .with(&mut foo.field_1, 5)
///     .with(&mut foo.field_2, 10)
///     .with(&mut foo.field_3, 15)
///     .set_all();
/// ```
///
/// # Safety
///
/// - ensure that all invariants are upheld after all assignments are complete
/// - you must not rely on the ordering of the assignments, that is, the Unit state should
///   be the same no matter the order of the assignments. This should be trivially verifiable,
///   since I'm pretty sure it's impossible. Just putting it in here in case someon can find
///   a way of doing this.
///
/// # Implementation Notes
///
/// This is possible to implement without storing references to the fields, but I don't think it
/// should matter in the Unit. This is probably optimised to the same thing? Not sure though.
/// I don't think it matters that much.
///
/// There's probably some kind of way of doing this with pure functions that inlines functions
/// more aggressively, as well.
pub struct SimultaneousUnsafeAssignment;

impl SimultaneousUnsafeAssignment {
    pub fn with<'b, Dst: UnsafeAssign<T>, T>(
        self,
        value: T,
        dst: &'b mut Dst,
    ) -> DeferredSimultaneousUnsafeAssignment<Self, DeferredUnsafeAssignment<'b, Dst, T>> {
        DeferredSimultaneousUnsafeAssignment {
            first: self,
            second: DeferredUnsafeAssignment { field: dst, value },
        }
    }
}

impl SimultaneousUnsafeAssign for SimultaneousUnsafeAssignment {
    unsafe fn set_all(self) {}
}

pub trait SimultaneousUnsafeAssign {
    /// Complete all assignments that have been deferred 'simultaneously'. This is not actually
    /// simultaneous, but ensures that all values are assigned, without the struct they are a
    /// part of being read in an invalid state
    ///
    /// # Safety
    /// - ensure that all invariants are upheld after all assignments are complete
    unsafe fn set_all(self);
}

pub struct DeferredSimultaneousUnsafeAssignment<
    First: SimultaneousUnsafeAssign,
    Second: SimultaneousUnsafeAssign,
> {
    first: First,
    second: Second,
}

impl<First: SimultaneousUnsafeAssign, Second: SimultaneousUnsafeAssign>
    DeferredSimultaneousUnsafeAssignment<First, Second>
{
    pub fn with<'b, Dst: UnsafeAssign<T>, T>(
        self,
        value: T,
        dst: &'b mut Dst,
    ) -> DeferredSimultaneousUnsafeAssignment<Self, DeferredUnsafeAssignment<'b, Dst, T>> {
        DeferredSimultaneousUnsafeAssignment {
            first: self,
            second: DeferredUnsafeAssignment { field: dst, value },
        }
    }
}

impl<'a, First: SimultaneousUnsafeAssign, Second: SimultaneousUnsafeAssign> SimultaneousUnsafeAssign
    for DeferredSimultaneousUnsafeAssignment<First, Second>
{
    unsafe fn set_all(self) {
        self.first.set_all();
        self.second.set_all();
    }
}

pub struct DeferredUnsafeAssignment<'a, Dst: UnsafeAssign<T>, T> {
    field: &'a mut Dst,
    value: T,
}

impl<'a, Dst: UnsafeAssign<T>, T> DeferredUnsafeAssignment<'a, Dst, T> {
    pub fn with<'b, UDst: UnsafeAssign<U>, U>(
        self,
        value: U,
        dst: &'b mut UDst,
    ) -> DeferredSimultaneousUnsafeAssignment<Self, DeferredUnsafeAssignment<'b, UDst, U>> {
        DeferredSimultaneousUnsafeAssignment {
            first: self,
            second: DeferredUnsafeAssignment { field: dst, value },
        }
    }
}

impl<'a, Dst: UnsafeAssign<T>, T> SimultaneousUnsafeAssign
    for DeferredUnsafeAssignment<'a, Dst, T>
{
    unsafe fn set_all(self) {
        self.field.set(self.value);
    }
}
