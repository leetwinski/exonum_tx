#[cfg(test)]
mod tx {
    use exonum_cryptocurrency_advanced::transactions::can_confirm_withdrawal;

    #[test]
    fn can_withdraw_with_sufficient_frozen() {
        assert!(can_confirm_withdrawal(100, 100, 10));
    }

    #[test]
    fn can_not_withdraw_with_insufficient_frozen() {
        assert!(!can_confirm_withdrawal(100, 2, 10));
    }

    #[test]
    fn can_not_withdraw_with_insufficient_original_balance() {
        assert!(!can_confirm_withdrawal(-100, 20, 10));
    }

    #[test]
    fn can_withdraw_with_sufficient_original_balance() {
        assert!(can_confirm_withdrawal(-100, 120, 10));
    }
}
