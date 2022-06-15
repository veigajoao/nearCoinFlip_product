use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_contract_state(&self) -> String {
        json!(&self).to_string()
    }

    pub fn view_partner_data(&self, nft_contract: AccountId) -> PartneredGame {
        self.games.get(&nft_contract).expect(ERR_002)
    }
}