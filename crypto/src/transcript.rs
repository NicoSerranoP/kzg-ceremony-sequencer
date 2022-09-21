use super::{CeremonyError, Contribution, Powers, G1, G2};
use crate::engine::Engine;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Transcript {
    #[serde(flatten)]
    pub powers: Powers,

    pub witness: Witness,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub struct Witness {
    #[serde(rename = "runningProducts")]
    pub products: Vec<G1>,

    #[serde(rename = "potPubkeys")]
    pub pubkeys: Vec<G2>,
}

impl Transcript {
    /// Create a new transcript for a ceremony of a given size.
    ///
    /// # Panics
    ///
    /// There must be at least two g1 and two g2 points, and there must be at
    /// least as many g1 as g2 points.
    #[must_use]
    pub fn new(num_g1: usize, num_g2: usize) -> Self {
        assert!(num_g1 >= 2);
        assert!(num_g2 >= 2);
        assert!(num_g1 >= num_g2);
        Self {
            powers:  Powers::new(num_g1, num_g2),
            witness: Witness::default(),
        }
    }

    /// Creates the start of a new contribution.
    #[must_use]
    pub fn contribution(&self) -> Contribution {
        Contribution {
            powers: self.powers.clone(),
            pubkey: G2::one(),
        }
    }

    /// Verifies a contribution.
    #[instrument(level = "info", skip_all, fields(n1=self.powers.g1.len(), n2=self.powers.g2.len()))]
    pub fn verify<E: Engine>(&self, contribution: &Contribution) -> Result<(), CeremonyError> {
        assert!(self.powers.g1.len() >= 2);
        assert!(self.powers.g2.len() >= 2);
        assert!(self.powers.g1.len() >= self.powers.g2.len());

        // Compatibility checks
        if self.powers.g1.len() != contribution.powers.g1.len() {
            return Err(CeremonyError::UnexpectedNumG1Powers(
                self.powers.g1.len(),
                contribution.powers.g1.len(),
            ));
        }
        if self.powers.g2.len() != contribution.powers.g2.len() {
            return Err(CeremonyError::UnexpectedNumG2Powers(
                self.powers.g2.len(),
                contribution.powers.g2.len(),
            ));
        }

        // Basic uniqueness checks.
        // All g1 powers must be unique. All g2 powers must be unique.
        // The pubkey must be unique in the pubkey set. The pubkey can not be in the g2
        // powers (except for the first contribution?).

        // TODO

        // Verify the contribution points (encoding and subgroup checks).
        E::validate_g1(&contribution.powers.g1)?;
        E::validate_g2(&contribution.powers.g2)?;
        E::validate_g2(&[contribution.pubkey])?;

        // Verify pairings.
        E::verify_pubkey(
            contribution.powers.g1[1],
            self.powers.g1[1],
            contribution.pubkey,
        )?;
        E::verify_g1(&contribution.powers.g1, contribution.powers.g2[1])?;
        E::verify_g2(
            &contribution.powers.g1[..contribution.powers.g2.len()],
            &contribution.powers.g2,
        )?;

        // Accept
        Ok(())
    }

    /// Adds a contribution to the transcaipt. The contribution must be
    /// verified.
    pub fn add(&mut self, contribution: Contribution) {
        self.witness.products.push(contribution.powers.g1[1]);
        self.witness.pubkeys.push(contribution.pubkey);
        self.powers = contribution.powers;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn transcript_json() {
        let t = Transcript::new(4, 2);
        let json = serde_json::to_value(&t).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
            "numG1Powers": 4,
            "numG2Powers": 2,
            "powersOfTau": {
                "G1Powers": [
                "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb"
                ],
                "G2Powers": [
                "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                "0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8",
                ]
            },
            "witness": {
                "runningProducts": [],
                "potPubkeys": []
            }
            })
        );
        let deser = serde_json::from_value::<Transcript>(json).unwrap();
        assert_eq!(deser, t);
    }
}