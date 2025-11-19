use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::StdRng,
};

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct WantsToPlay {
    pub id: usize,
    pub elo: i32,
    pub variant: &'static str,
    pub time_control: &'static str,
}

impl WantsToPlay {
    pub fn new(id: usize, elo: i32, variant: &'static str, time_control: &'static str) -> Self {
        WantsToPlay {
            id,
            elo,
            variant,
            time_control,
        }
    }
}

#[macro_export]
macro_rules! declare_chess {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[WantsToPlay]) -> Vec<test_suite::MatchTrace<WantsToPlay>> {
            let mut mailbox: fairjax_core::MailBox<WantsToPlay> = fairjax_core::MailBox::default();
            let mut output = vec![];

            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, WantsToPlay] {
                    #[$strategy]
                    (
                        WantsToPlay {
                            id: id1,
                            elo: elo1,
                            variant: variant1,
                            time_control: time_control1,
                        },
                        WantsToPlay {
                            id: id2,
                            elo: elo2,
                            variant: variant2,
                            time_control: time_control2,
                        },
                    ) if (elo1 - elo2).abs() < 200
                        && variant1 == variant2
                        && time_control1 == time_control2 =>
                    {
                        output.push(test_suite::MatchTrace::new(
                            0,
                            vec![
                                WantsToPlay::new(id1, elo1, variant1, time_control1),
                                WantsToPlay::new(id2, elo2, variant2, time_control2),
                            ],
                        ));
                    }
                });
            }
            output
        }
    };
}

#[macro_export]
macro_rules! partitions_declare_chess {
    ($fn_name:ident) => {
        fn $fn_name(messages: &[WantsToPlay]) -> Vec<test_suite::MatchTrace<WantsToPlay>> {
            let mut mailbox: fairjax_core::MailBox<WantsToPlay> = fairjax_core::MailBox::default();
            let mut output = vec![];

            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, WantsToPlay] {
                    (
                        WantsToPlay {
                            id: id1,
                            elo: elo1,
                            variant,
                            time_control,
                        },
                        WantsToPlay {
                            id: id2,
                            elo: elo2,
                            variant,
                            time_control,
                        },
                    ) if (elo1 - elo2).abs() < 200 => {
                        output.push(test_suite::MatchTrace::new(
                            0,
                            vec![
                                WantsToPlay::new(id1, elo1, variant, time_control),
                                WantsToPlay::new(id2, elo2, variant, time_control),
                            ],
                        ));
                    }
                });
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize, seed: Option<u64>) -> Vec<WantsToPlay> {
    let mut rng = crate::get_rng(seed);
    let messages: Vec<_> = (0..size)
        .map(|i| WantsToPlay {
            id: i,
            elo: rng.random_range(500..2000),
            variant: random_variant(&mut rng),
            time_control: random_time_control(&mut rng),
        })
        .collect();
    messages
}

fn random_variant(rng: &mut StdRng) -> &'static str {
    let weights = [0.7, 0.15, 0.1, 0.05];
    let outcomes = ["standard", "chess960", "3check", "atomic"];

    let dist = WeightedIndex::new(&weights).unwrap();

    outcomes[dist.sample(rng)]
}

fn random_time_control(rng: &mut StdRng) -> &'static str {
    let weights = [0.5, 0.2, 0.1, 0.09, 0.07, 0.04];
    let outcomes = ["5+0", "3+0", "10+0", "3+2", "5+5", "1+0"];

    let dist = WeightedIndex::new(&weights).unwrap();

    outcomes[dist.sample(rng)]
}
