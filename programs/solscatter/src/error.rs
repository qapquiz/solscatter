use anchor_lang::prelude::*;

#[error_code]
pub enum SolscatterError {
    #[msg("number of rewards in drawing result must be greater than 0")]
    NumberOfRewardsMustMoreThanZero,
    NumberOfRewardsMustLessOrEqualTen,
    #[msg("number of random numbers must equal to number of rewards")]
    NumberOfRandomNumbersNotMatchWithNumberOfRewards,
    InvalidSwitchboardVrfAccount,
    #[msg("Invalid collateral mint")]
    InvalidCollateralMint,
    #[msg("Invalid collateral owner")]
    InvalidCollateralOwner,
    #[msg("Insufficient amount")]
    InsufficientAmount
}