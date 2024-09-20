use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    hash::Hash,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref,
        DerefMut, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Not, Shl, ShlAssign, Shr,
        ShrAssign, Sub, SubAssign,
    },
};

/// A boolean value that can be used to control the flow inside of [`Application::sync`].
///
/// Defaults to a value of `true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncBool(RefCell<bool>);

impl Default for SyncBool {
    fn default() -> Self {
        Self::new(true)
    }
}

impl SyncBool {
    pub fn new(value: bool) -> Self {
        Self(RefCell::new(value))
    }

    /// Set the value
    pub fn set(&self, value: bool) {
        *RefCell::borrow_mut(&self.0) = value;
    }

    /// Set the value to `false` and return the old value.
    #[inline]
    pub fn get(&mut self) -> bool {
        let b = RefCell::get_mut(&mut self.0);
        let value = *b;
        *b = false;
        value
    }
}

impl From<bool> for SyncBool {
    fn from(value: bool) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a mut SyncBool> for bool {
    fn from(value: &'a mut SyncBool) -> Self {
        value.get()
    }
}

/// A wrapper around a data type that automatically detects when the data has been
/// borrowed mutably. Useful for easy and performant diffing inside of
/// [`Application::sync`].
///
/// [`Application::sync`]: ../crate/application/struct.Application.html
pub struct Diff<T> {
    data: T,
    changed: RefCell<bool>,
}

impl<T> Diff<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data,
            changed: RefCell::new(true),
        }
    }

    pub fn into_inner(self) -> T {
        self.data
    }

    /// Returns `true` if the data has been borrowed mutably since the last call
    /// to [`Diff::changed`]
    pub fn changed(&self) -> bool {
        let mut b = RefCell::borrow_mut(&self.changed);
        let value = *b;
        *b = false;
        value
    }
}

impl<T> Deref for Diff<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Diff<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        *RefCell::get_mut(&mut self.changed) = true;
        &mut self.data
    }
}

impl<T> From<T> for Diff<T> {
    fn from(data: T) -> Self {
        Diff::new(data)
    }
}

impl<T> Clone for Diff<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            changed: self.changed.clone(),
        }
    }
}

impl<T> PartialEq<T> for Diff<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, rhs: &T) -> bool {
        self.data.eq(rhs)
    }
}

impl<T> PartialEq for Diff<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.data.eq(&rhs.data)
    }
}

impl<T> Eq for Diff<T> where T: PartialEq + Eq {}

impl<T> Debug for Diff<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<T> Display for Diff<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<T> PartialOrd<T> for Diff<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other)
    }
}

impl<T> PartialOrd for Diff<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T> Ord for Diff<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl<T> Hash for Diff<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

impl<T> Add<T> for Diff<T>
where
    T: Add<Output = T>,
{
    type Output = T;
    fn add(self, rhs: T) -> Self::Output {
        self.data.add(rhs)
    }
}

impl<T> Sub<T> for Diff<T>
where
    T: Sub<Output = T>,
{
    type Output = T;
    fn sub(self, rhs: T) -> Self::Output {
        self.data.sub(rhs)
    }
}

impl<T> Mul<T> for Diff<T>
where
    T: Mul<Output = T>,
{
    type Output = T;
    fn mul(self, rhs: T) -> Self::Output {
        self.data.mul(rhs)
    }
}

impl<T> Div<T> for Diff<T>
where
    T: Div<Output = T>,
{
    type Output = T;
    fn div(self, rhs: T) -> Self::Output {
        self.data.div(rhs)
    }
}

impl<T> Not for Diff<T>
where
    T: Not<Output = T>,
{
    type Output = T;
    fn not(self) -> Self::Output {
        self.data.not()
    }
}

impl<T> BitAnd<T> for Diff<T>
where
    T: BitAnd<Output = T>,
{
    type Output = T;
    fn bitand(self, rhs: T) -> Self::Output {
        self.data.bitand(rhs)
    }
}

impl<T> BitOr<T> for Diff<T>
where
    T: BitOr<Output = T>,
{
    type Output = T;
    fn bitor(self, rhs: T) -> Self::Output {
        self.data.bitor(rhs)
    }
}

impl<T> BitXor<T> for Diff<T>
where
    T: BitXor<Output = T>,
{
    type Output = T;
    fn bitxor(self, rhs: T) -> Self::Output {
        self.data.bitxor(rhs)
    }
}

impl<T> Shl<T> for Diff<T>
where
    T: Shl<Output = T>,
{
    type Output = T;
    fn shl(self, rhs: T) -> Self::Output {
        self.data.shl(rhs)
    }
}

impl<T> Shr<T> for Diff<T>
where
    T: Shr<Output = T>,
{
    type Output = T;
    fn shr(self, rhs: T) -> Self::Output {
        self.data.shr(rhs)
    }
}

impl<T> AddAssign<T> for Diff<T>
where
    T: AddAssign<T>,
{
    fn add_assign(&mut self, rhs: T) {
        self.deref_mut().add_assign(rhs)
    }
}

impl<T> SubAssign<T> for Diff<T>
where
    T: SubAssign<T>,
{
    fn sub_assign(&mut self, rhs: T) {
        self.deref_mut().sub_assign(rhs)
    }
}

impl<T> MulAssign<T> for Diff<T>
where
    T: MulAssign<T>,
{
    fn mul_assign(&mut self, rhs: T) {
        self.deref_mut().mul_assign(rhs)
    }
}

impl<T> DivAssign<T> for Diff<T>
where
    T: DivAssign<T>,
{
    fn div_assign(&mut self, rhs: T) {
        self.deref_mut().div_assign(rhs)
    }
}

impl<T> BitAndAssign<T> for Diff<T>
where
    T: BitAndAssign<T>,
{
    fn bitand_assign(&mut self, rhs: T) {
        self.deref_mut().bitand_assign(rhs)
    }
}

impl<T> BitOrAssign<T> for Diff<T>
where
    T: BitOrAssign<T>,
{
    fn bitor_assign(&mut self, rhs: T) {
        self.deref_mut().bitor_assign(rhs)
    }
}

impl<T> BitXorAssign<T> for Diff<T>
where
    T: BitXorAssign<T>,
{
    fn bitxor_assign(&mut self, rhs: T) {
        self.deref_mut().bitxor_assign(rhs)
    }
}

impl<T> ShlAssign<T> for Diff<T>
where
    T: ShlAssign<T>,
{
    fn shl_assign(&mut self, rhs: T) {
        self.deref_mut().shl_assign(rhs)
    }
}

impl<T> ShrAssign<T> for Diff<T>
where
    T: ShrAssign<T>,
{
    fn shr_assign(&mut self, rhs: T) {
        self.deref_mut().shr_assign(rhs)
    }
}

impl<T, Idx> Index<Idx> for Diff<T>
where
    T: Index<Idx>,
{
    type Output = <T as Index<Idx>>::Output;
    fn index(&self, index: Idx) -> &Self::Output {
        self.data.index(index)
    }
}

impl<T, Idx> IndexMut<Idx> for Diff<T>
where
    T: IndexMut<Idx>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.deref_mut().index_mut(index)
    }
}
