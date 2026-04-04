/// Client HTTP — appelle le serveur Axum du PC serveur.
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::db::{
    AppError, Contribution, ContributionEditInput, ContributionInput, ContributionWithMember,
    Member, MemberInput, MemberWithTotal, YearSummary,
};

pub struct RemoteClient {
    pub base_url: String,
    client: Client,
}

impl RemoteClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url, client: Client::new() }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check_response(resp: Response) -> Result<Response, AppError> {
        if resp.status().is_success() {
            Ok(resp)
        } else {
            let msg = resp.text().await.unwrap_or_else(|_| "Erreur inconnue".to_string());
            Err(AppError::Network(format!("Erreur serveur : {msg}")))
        }
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        let resp = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Connexion échouée : {e}")))?;

        Self::check_response(resp)
            .await?
            .json::<T>()
            .await
            .map_err(|e| AppError::Network(format!("Réponse invalide : {e}")))
    }

    async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let resp = self
            .client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Connexion échouée : {e}")))?;

        Self::check_response(resp)
            .await?
            .json::<T>()
            .await
            .map_err(|e| AppError::Network(format!("Réponse invalide : {e}")))
    }

    async fn put_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let resp = self
            .client
            .put(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Connexion échouée : {e}")))?;

        Self::check_response(resp)
            .await?
            .json::<T>()
            .await
            .map_err(|e| AppError::Network(format!("Réponse invalide : {e}")))
    }

    async fn delete_req(&self, path: &str) -> Result<(), AppError> {
        let resp = self
            .client
            .delete(self.url(path))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Connexion échouée : {e}")))?;

        Self::check_response(resp).await.map(|_| ())
    }

    async fn get_bytes(&self, path: &str) -> Result<Vec<u8>, AppError> {
        let resp = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Connexion échouée : {e}")))?;

        Self::check_response(resp)
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| AppError::Network(format!("Erreur lecture bytes : {e}")))
    }

    // ── Members ───────────────────────────────────────────────────────────────

    pub async fn get_members(&self) -> Result<Vec<Member>, AppError> {
        self.get_json("/api/members").await
    }

    pub async fn get_members_by_type(&self, member_type: &str) -> Result<Vec<Member>, AppError> {
        self.get_json(&format!("/api/members/by-type/{member_type}")).await
    }

    pub async fn get_members_by_type_with_total(
        &self,
        member_type: &str,
    ) -> Result<Vec<MemberWithTotal>, AppError> {
        self.get_json(&format!("/api/members/by-type/{member_type}/totals")).await
    }

    pub async fn get_member(&self, id: i64) -> Result<Member, AppError> {
        self.get_json(&format!("/api/members/{id}")).await
    }

    pub async fn create_member(&self, input: MemberInput) -> Result<Member, AppError> {
        self.post_json("/api/members", &input).await
    }

    pub async fn update_member(&self, id: i64, input: MemberInput) -> Result<Member, AppError> {
        self.put_json(&format!("/api/members/{id}"), &input).await
    }

    pub async fn delete_member(&self, id: i64) -> Result<(), AppError> {
        self.delete_req(&format!("/api/members/{id}")).await
    }

    pub async fn transfer_members(&self, ids: &[i64], new_type: &str) -> Result<usize, AppError> {
        #[derive(Serialize)]
        struct Body<'a> { ids: &'a [i64], new_type: &'a str }
        self.post_json("/api/transfer-members", &Body { ids, new_type }).await
    }

    // ── Contributions ─────────────────────────────────────────────────────────

    pub async fn get_contributions(&self, member_id: i64) -> Result<Vec<Contribution>, AppError> {
        self.get_json(&format!("/api/contributions/by-member/{member_id}")).await
    }

    pub async fn get_contributions_by_year(&self, year: i32) -> Result<Vec<Contribution>, AppError> {
        self.get_json(&format!("/api/contributions/by-year/{year}")).await
    }

    pub async fn create_contribution(&self, input: ContributionInput) -> Result<Contribution, AppError> {
        self.post_json("/api/contributions", &input).await
    }

    pub async fn delete_contribution(&self, id: i64) -> Result<(), AppError> {
        self.delete_req(&format!("/api/contributions/{id}")).await
    }

    pub async fn get_contributions_by_year_with_member(
        &self,
        year: i32,
    ) -> Result<Vec<ContributionWithMember>, AppError> {
        self.get_json(&format!("/api/contributions/by-year/{year}/with-member")).await
    }

    pub async fn get_all_contributions_with_member(
        &self,
    ) -> Result<Vec<ContributionWithMember>, AppError> {
        self.get_json("/api/contributions/all/with-member").await
    }

    // ── Year Summaries ────────────────────────────────────────────────────────

    pub async fn get_year_summaries(&self) -> Result<Vec<YearSummary>, AppError> {
        self.get_json("/api/year-summaries").await
    }

    pub async fn get_year_summary(&self, year: i32) -> Result<Option<YearSummary>, AppError> {
        self.get_json(&format!("/api/year-summaries/{year}")).await
    }

    pub async fn close_year(&self, year: i32, note: Option<String>) -> Result<YearSummary, AppError> {
        #[derive(Serialize)]
        struct Body { note: Option<String> }
        self.post_json(&format!("/api/year-summaries/{year}/close"), &Body { note }).await
    }

    pub async fn reopen_year(&self, year: i32) -> Result<YearSummary, AppError> {
        self.post_json(&format!("/api/year-summaries/{year}/reopen"), &serde_json::json!({})).await
    }

    pub async fn check_and_close_previous_year(&self) -> Result<Option<YearSummary>, AppError> {
        self.post_json("/api/year/check-close", &serde_json::json!({})).await
    }

    // ── PIN ───────────────────────────────────────────────────────────────────

    pub async fn set_pin(&self, _pin: &str) -> Result<(), AppError> {
        // Le PIN ne se configure que sur le serveur local ; le client ne peut pas l'appeler.
        Err(AppError::Validation("Le PIN ne peut être configuré que sur le serveur.".into()))
    }

    pub async fn verify_pin(&self, pin: &str) -> Result<bool, AppError> {
        #[derive(Serialize)]
        struct Body<'a> { pin: &'a str }
        self.post_json("/api/verify-pin", &Body { pin }).await
    }

    pub async fn update_contribution(
        &self,
        id: i64,
        input: ContributionEditInput,
    ) -> Result<Contribution, AppError> {
        self.put_json(&format!("/api/contributions/{id}"), &input).await
    }

    // ── Export / Import ───────────────────────────────────────────────────────

    pub async fn export_members_csv(&self, member_type: &str) -> Result<String, AppError> {
        self.get_json::<String>(&format!("/api/export/csv/{member_type}")).await
    }

    pub async fn export_members_excel(&self, member_type: &str) -> Result<Vec<u8>, AppError> {
        self.get_bytes(&format!("/api/export/excel/{member_type}")).await
    }

    pub async fn import_members_csv(
        &self,
        csv_content: String,
        member_type: &str,
    ) -> Result<usize, AppError> {
        #[derive(Serialize)]
        struct Body { content: String }
        self.post_json(&format!("/api/import/csv/{member_type}"), &Body { content: csv_content }).await
    }
}
