use serde::Deserialize;

#[derive(Copy, Debug, Clone, Hash, Ord, Eq, PartialEq, PartialOrd, Default, Deserialize)]
pub enum RegEnum {
    #[default]
    A,
    X,
    Y,
    PC,
    S,
    P,
}
