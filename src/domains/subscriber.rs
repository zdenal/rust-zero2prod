use derive_getters::Getters;
use validator::{Validate, ValidationError, ValidationErrors};

fn is_valid_name(name: &str) -> Result<(), ValidationError> {
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    if name.chars().any(|ref ch| forbidden_characters.contains(ch)) {
        return Err(ValidationError::new("contains"));
    };

    Ok(())
}

#[derive(Debug, Validate, Getters)]
pub struct NewSubscriber {
    #[validate(length(max = 256, min = 1), custom = "is_valid_name")]
    name: String,
    #[validate(email)]
    email: String,
}

impl NewSubscriber {
    pub fn parse(name: &str, email: &str) -> Result<Self, ValidationErrors> {
        let s = Self {
            name: name.to_string(),
            email: email.to_string(),
        };
        match s.validate() {
            Ok(_) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_ok;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use rand::SeedableRng;
    use validator::ValidationErrors;

    use crate::domains::subscriber::NewSubscriber;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            //let mut rng = rand::rngs::std::StdRng::seed_from_u64(u64::arbitrary(g));
            let mut rng = rand::rngs::StdRng::seed_from_u64(u64::arbitrary(g));

            Self(SafeEmail().fake_with_rng(&mut rng))
        }
    }

    impl AsRef<str> for ValidEmailFixture {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    #[test]
    fn valid_case() {
        assert_ok!(NewSubscriber::parse("test", "test@test.com",));
    }

    #[test]
    fn long_name() {
        match NewSubscriber::parse("t".repeat(257).as_ref(), "test@test.com") {
            Ok(_) => panic!("NewSubscriber parsing error"),
            Err(e) => {
                assert!(ValidationErrors::has_error(&Err(e.clone()), "name"));
                assert!(!ValidationErrors::has_error(&Err(e), "email"));
            }
        }
    }

    #[test]
    fn forbidden_char_in_name_is_rejected() {
        match NewSubscriber::parse("test(t", "test@test.com") {
            Ok(_) => panic!("NewSubscriber parsing error"),
            Err(e) => assert!(ValidationErrors::has_error(&Err(e), "name")),
        }
    }

    #[test]
    fn invalid_email() {
        match NewSubscriber::parse("test", "test@.com") {
            Ok(_) => panic!("NewSubscriber parsing error"),
            Err(e) => {
                assert!(ValidationErrors::has_error(&Err(e.clone()), "email"));
                assert!(!ValidationErrors::has_error(&Err(e), "name"));
            }
        }
    }

    #[quickcheck_macros::quickcheck]
    fn property_test_on_email(valid_email: ValidEmailFixture) -> bool {
        NewSubscriber::parse("name", valid_email.as_ref()).is_ok()
    }
}
