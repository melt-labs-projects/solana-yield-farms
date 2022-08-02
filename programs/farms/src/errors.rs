use anchor_lang::prelude::*;
use anchor_lang::error;

#[error]
pub enum FarmError {

    #[msg("Only the program owner can use this endpoint")]
    OnlyOwner,

    #[msg("The amount cannot be zero")]
    AmountIsZero,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Numerical overflow error")]
    NumericalOverflowError,

    #[msg("Fee is too large")]
    InvalidFee,

    #[msg("Amount is too large")]
    AmountIsTooLarge,

}