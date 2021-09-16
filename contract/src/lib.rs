use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::ValidAccountId;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, BorshStorageKey, PanicOnDefault};

setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct VoteSmart {
    master_account_id: AccountId,
    parties: UnorderedMap<u64, String>,
    campaigns: UnorderedMap<u64, String>,
    regions: UnorderedMap<u64, Region>,
    districts: UnorderedMap<u64, District>,
    candidates: UnorderedMap<u64, Candidate>,
    recommendations: LookupMap<RecommendationIndex, u64>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Region {
    pub title: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct District {
    pub region_id: u64,
    pub title: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Candidate {
    pub title: String,
    pub party_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Party {
    pub index: u64,
    pub title: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Recommendation {
    pub title: String,
    pub party: String,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RecommendationIndex {
    pub campaign_id: u64,
    pub district_id: u64,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Parties,
    Campaigns,
    Regions,
    Districts,
    Candidates,
    Recommendations,
}

#[near_bindgen]
impl VoteSmart {
    #[init]
    pub fn new(admin_id: Option<ValidAccountId>) -> Self {
        let master_account_id: AccountId = if let Some(account_id) = admin_id {
            account_id.into()
        } else {
            env::predecessor_account_id()
        };

        Self {
            master_account_id,
            parties: UnorderedMap::new(StorageKey::Parties),
            campaigns: UnorderedMap::new(StorageKey::Campaigns),
            regions: UnorderedMap::new(StorageKey::Regions),
            districts: UnorderedMap::new(StorageKey::Districts),
            candidates: UnorderedMap::new(StorageKey::Candidates),
            recommendations: LookupMap::new(StorageKey::Recommendations),
        }
    }

    pub(crate) fn assert_access(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.master_account_id,
            "No access"
        );
    }

    pub fn set_master_account_id(&mut self, admin_id: ValidAccountId) {
        self.assert_access();
        self.master_account_id = admin_id.into();
    }

    pub fn add_campaign(&mut self, id: u64, title: String) {
        self.assert_access();
        self.campaigns.insert(&id, &title);
    }

    pub fn get_campaigns(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(u64, String)> {
        unordered_map_pagination(&self.campaigns, from_index, limit)
    }

    pub fn add_parties(&mut self, parties: Vec<(u64, String)>) {
        self.assert_access();
        for data in parties {
            self.parties.insert(&data.0, &data.1);
        }
    }

    pub fn get_parties(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(u64, String)> {
        unordered_map_pagination(&self.parties, from_index, limit)
    }

    pub fn add_regions(&mut self, regions: Vec<(u64, Region)>) {
        self.assert_access();
        for data in regions {
            self.regions.insert(&data.0, &data.1);
        }
    }

    pub fn get_regions(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(u64, Region)> {
        unordered_map_pagination(&self.regions, from_index, limit)
    }

    pub fn add_districts(&mut self, districts: Vec<(u64, District)>) {
        self.assert_access();
        for data in districts {
            self.districts.insert(&data.0, &data.1);
        }
    }

    pub fn get_districts(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(u64, District)> {
        unordered_map_pagination(&self.districts, from_index, limit)
    }

    pub fn get_districts_by_region(
        &self,
        region_id: u64,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(u64, District)> {
        let keys = self.districts.keys_as_vector();
        let values = self.districts.values_as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(keys.len());
        (from_index..std::cmp::min(keys.len(), limit))
            .filter(|index| values.get(*index).unwrap().region_id == region_id)
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap().into()))
            .collect()
    }

    pub fn add_candidates(&mut self, candidates: Vec<(u64, Candidate)>) {
        self.assert_access();
        for data in candidates {
            self.candidates.insert(&data.0, &data.1);
        }
    }

    pub fn get_candidates(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(u64, Candidate)> {
        unordered_map_pagination(&self.candidates, from_index, limit)
    }

    // recommendations: [campaign_id: u64, district_id: u64, candidate_id: u64]
    pub fn add_recommendations(&mut self, recommendations: Vec<(u64, u64, u64)>) {
        self.assert_access();

        for data in recommendations {
            let campaign_id = data.0;
            let district_id = data.1;
            let candidate_id = data.2;

            self.recommendations.insert(
                &RecommendationIndex {
                    campaign_id,
                    district_id,
                },
                &candidate_id,
            );
        }
    }

    pub fn get_votesmart(&self, campaign_id: u64, district_id: u64) -> Option<Recommendation> {
        let candidate_id = self.recommendations.get(&RecommendationIndex {
            campaign_id,
            district_id,
        });

        if let Some(candidate_id_unwrapped) = candidate_id {
            if let Some(candidate_unwrapped) = self.candidates.get(&candidate_id_unwrapped) {
                let result = Recommendation {
                    title: candidate_unwrapped.title,
                    party: self
                        .parties
                        .get(&candidate_unwrapped.party_id)
                        .unwrap_or("Unknown".to_string()),
                };
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub(crate) fn unordered_map_pagination<K, VV, V>(
    m: &UnorderedMap<K, VV>,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> Vec<(K, V)>
where
    K: BorshSerialize + BorshDeserialize,
    VV: BorshSerialize + BorshDeserialize,
    V: From<VV>,
{
    let keys = m.keys_as_vector();
    let values = m.values_as_vector();
    let from_index = from_index.unwrap_or(0);
    let limit = limit.unwrap_or(keys.len());
    (from_index..std::cmp::min(keys.len(), limit))
        .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap().into()))
        .collect()
}
