use fixed::traits::FromFixed;
use fixed::traits::ToFixed;
use fixed::types::I16F16 as InternalType;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FractionalType(InternalType);

impl FractionalType
{
    pub fn from_fixed(fixed: InternalType) -> Self
    {
        Self(fixed)
    }

    pub fn from_num<T: ToFixed>(num: T) -> Self
    {
        Self(InternalType::from_num(num))
    }

    pub fn to_num<T: FromFixed>(&self) -> T
    {
        self.0.to_num()
    }

    pub fn round(&self) -> Self
    {
        Self(self.0.round())
    }

    pub fn floor(&self) -> Self
    {
        Self(self.0.floor())
    }

    pub fn ceil(&self) -> Self
    {
        Self(self.0.ceil())
    }
}

impl std::ops::Add for FractionalType
{
    type Output = Self;

    fn add(self, other: Self) -> Self
    {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for FractionalType
{
    type Output = Self;

    fn sub(self, other: Self) -> Self
    {
        Self(self.0 - other.0)
    }
}

impl std::ops::Mul for FractionalType
{
    type Output = Self;

    fn mul(self, other: Self) -> Self
    {
        Self(self.0 * other.0)
    }
}

impl std::ops::Div for FractionalType
{
    type Output = Self;

    fn div(self, other: Self) -> Self
    {
        Self(self.0 / other.0)
    }
}

impl num_traits::identities::Zero for FractionalType
{

    fn zero() -> Self
    {
        Self(InternalType::from_num(0))
    }

    fn is_zero(&self) -> bool
    {
        let z = Self::zero();
        self == &z
    }
}