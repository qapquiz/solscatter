use anchor_lang::prelude::*;

#[error_code]
pub enum SolscatterError {
    #[msg("number of rewards in drawing result must be greater than 0")]
    NumberOfRewardsMustMoreThanZero,
    NumberOfRewardsMustLessOrEqualTen,
    #[msg("number of random numbers must equal to number of rewards")]
    NumberOfRandomNumbersNotMatchWithNumberOfRewards,
    InvalidSwitchboardVrfAccount,
    #[msg("Invalid reserve liquidity supply")]
    InvalidReserveLiquiditySupply,
    #[msg("Invalid reserve mint")]
    InvalidReserveCollateralMint,
    #[msg("Invalid lending market")]
    InvalidLendingMarket,
    #[msg("Invalid source liquidity mint")]
    InvalidSourceLiquidity,
    #[msg("Invalid collateral mint")]
    InvalidCollateralMint,
    #[msg("Invalid collateral supply")]
    InvalidCollateralSupply,
    #[msg("Invalid collateral owner")]
    InvalidCollateralOwner,
    #[msg("Invalid pyth oracle")]
    InvalidPythOracle,
    #[msg("Invalid switchboard oracle")]
    InvalidSwitchboardOracle,
    #[msg("Invalid c token pyth oracle")]
    InvalidCTokenPythOracle,
    #[msg("Invalid c token switchboard oracle")]
    InvalidCTokenSwitchboardOracle,
    #[msg("Insufficient amount")]
    InsufficientAmount
}