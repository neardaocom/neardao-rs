#[cfg(test)]
mod test {
    use crate::{create_val_to_percent_closure, release::ReleaseModel, unit_tests::DURATION_1Y_S};

    #[test]
    fn release() {
        let release_model = ReleaseModel::None;
        let released = release_model.release(9999, 0, 1, 0);
        assert_eq!(released, 0);

        let release_model = ReleaseModel::Linear {
            duration: DURATION_1Y_S,
            release_end: DURATION_1Y_S,
        };

        let perc_to_val = create_val_to_percent_closure!(DURATION_1Y_S, u32);


        let released = release_model.release(1000, 10, 0, perc_to_val(0));
        assert_eq!(released, 0);

        let released = release_model.release(1000, 10, 0, perc_to_val(1));
        assert_eq!(released, 9);

        let released = release_model.release(1000, 10, 0, perc_to_val(2));
        assert_eq!(released, 19);

        let released = release_model.release(1000, 10, 0, perc_to_val(5));
        assert_eq!(released, 49);

        let released = release_model.release(1000, 10, 0, perc_to_val(6));
        assert_eq!(released, 59);

        let released = release_model.release(1000, 10, 0, perc_to_val(7));
        assert_eq!(released, 69);

        let released = release_model.release(1000, 10, 0, perc_to_val(10));
        assert_eq!(released, 99);

        let released = release_model.release(1000, 10, 0, perc_to_val(99));
        assert_eq!(released, 980);

        let released = release_model.release(1000, 10, 0, perc_to_val(100));
        assert_eq!(released, 990);

        let released = release_model.release(1000, 10, 0, perc_to_val(101));
        assert_eq!(released, 990);
    }
}
