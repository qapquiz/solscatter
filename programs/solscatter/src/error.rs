use anchor_lang::prelude::*;

#[error_code]
pub enum SolscatterError {
    #[msg("number of rewards in drawing result must be greater than 0")]
    NumberOfRewardsMustMoreThanZero,
    #[msg("number of rewards in drawing reulst must be less than or equal 10")]
    NumberOfRewardsMustLessOrEqualTen,
    #[msg("number of random numbers must equal to number of rewards")]
    NumberOfRandomNumbersNotMatchWithNumberOfRewards,
    #[msg("wrong switchboard vrf account")]
    InvalidSwitchboardVrfAccount,
    #[msg("insufficient balance")]
    InsufficientBalance,
}