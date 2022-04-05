use anchor_lang::prelude::*;
use solana_maths::{Decimal, TryMul, TryDiv};

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Fee {
    pub bips: u16,
}

impl Fee {
    pub fn calculate_fee(&self, principle_amount: u64) -> Result<u64> {
        if principle_amount == 0 {
            return Ok(0);
        }

        return Ok(Decimal::from(principle_amount)
            .try_mul(self.bips as u64)?
            .try_div(10_000)?
            .try_ceil_u64()?
        );
    }

    pub fn calculate_fee_with_discount_peroid(
        &self,
        principle_amount: u64,
        discount_period: u64,
        start_calculate_timestamp: i64,
        current_timestamp: i64
    ) -> Result<u64> {
        if principle_amount == 0 {
            return Ok(0);
        }

        require_gt!(start_calculate_timestamp, 0);
        require_gt!(current_timestamp, 0);

        if start_calculate_timestamp == current_timestamp {
            // max fee
            return Ok(self.calculate_fee(principle_amount)?);
        }

        let time_at_zero_fee = start_calculate_timestamp.checked_add(discount_period as i64).unwrap();
        if time_at_zero_fee < current_timestamp {
            // no fee 
            return Ok(0);
        }

        // fee with discount
        let delta_time = time_at_zero_fee.checked_sub(current_timestamp).unwrap();
        let discount_percentage_in_bips = Decimal::from(delta_time as u64)
            .try_mul(10_000)?
            .try_div(discount_period)?;
        let final_fee_in_bips = Decimal::from(self.bips as u64)
            .try_mul(discount_percentage_in_bips)?
            .try_div(10_000)?;

        return Ok(Decimal::from(principle_amount)
            .try_mul(final_fee_in_bips)?
            .try_div(10_000)?
            .try_ceil_u64()?
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::state::fee::Fee;

    #[test]
    fn calculate_fee_when_principle_amount_is_zero() {
        let five_percent_fee = Fee { bips: 500 }; // 5%
        match five_percent_fee.calculate_fee(0) {
            Ok(fee_amount) => {
                assert_eq!(fee_amount, 0);
            },
            Err(_) => { assert!(false); }
        };

        match five_percent_fee.calculate_fee_with_discount_peroid(0, 100, 100, 100) {
            Ok(fee_amount) => { assert_eq!(fee_amount, 0); },
            Err(_) => { assert!(false); }
        }
    }

    #[test]
    fn calculate_fee() {
        let principle_amount = 1_000_000;
        let five_percent_fee = Fee { bips: 500 }; // 5%
        match five_percent_fee.calculate_fee(principle_amount) {
            Ok(fee_amount) => {
                assert_eq!(fee_amount, 50_000);
            },
            Err(_) => { assert!(false); }
        }
    }

    #[test]
    fn calculate_fee_20_percent_period() {
        let principle_amount = 1_000_000;
        let discount_period = 5;
        let start_calculate_timestamp = 10;
        let current_timestamp = 11;
        let five_percent_fee = Fee { bips: 500 }; // 5%
        match five_percent_fee.calculate_fee_with_discount_peroid(
            principle_amount,
            discount_period,
            start_calculate_timestamp,
            current_timestamp
        ) {
            Ok(fee_amount) => {
                assert_eq!(fee_amount, 40_000);
            },
            Err(_) => { assert!(false); },
        }
    }

    #[test]
    fn calculate_fee_27_percent_period() {
        let principle_amount = 1_000_000;
        let discount_period = 100;
        let start_calculate_timestamp = 100;
        let current_timestamp = 127;
        let five_percent_fee = Fee { bips: 500 }; // 5%
        match five_percent_fee.calculate_fee_with_discount_peroid(
            principle_amount,
            discount_period,
            start_calculate_timestamp,
            current_timestamp,
        ) {
            Ok(fee_amount) => { assert_eq!(fee_amount, 36_500) },
            Err(_) => { assert!(false) },
        }
    }

    #[test]
    fn calculate_fee_40_percent_period() {
        let principle_amount = 1_111_111;
        let discount_period = 5;
        let start_calculate_timestamp = 10;
        let current_timestamp = 12;
        let five_percent_fee = Fee { bips: 500 }; // 5%
        match five_percent_fee.calculate_fee_with_discount_peroid(
            principle_amount,
            discount_period,
            start_calculate_timestamp,
            current_timestamp
        ) {
            Ok(fee_amount) => {
                assert_eq!(fee_amount, 33_334);
            },
            Err(_) => { assert!(false); },
        }
    }
}
