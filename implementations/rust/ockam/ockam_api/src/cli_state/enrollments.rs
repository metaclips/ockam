use crate::authenticator::one_time_code::OneTimeCode;
use crate::cli_state::Result;
use crate::cli_state::{CliState, CliStateError};
use crate::colors::{color_ok, color_primary, color_warn};
use crate::error::ApiError;
use crate::orchestrator::email_address::EmailAddress;
use crate::orchestrator::project::models::ProjectModel;
use crate::output::human_readable_time;
use crate::terminal::fmt;
use ockam::identity::{Identifier, Identity, TimestampInSeconds, Vault};
use ockam_multiaddr::proto::Service;
use ockam_multiaddr::{MultiAddr, Protocol};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use time::OffsetDateTime;

/// The following CliState methods help keeping track of
/// the enrollment status of identities.
impl CliState {
    #[instrument(skip_all, fields(name = name.clone()))]
    pub async fn is_identity_enrolled(&self, name: &Option<String>) -> Result<bool> {
        let repository = self.enrollment_repository();

        match name {
            Some(name) => Ok(repository.is_identity_enrolled(name).await?),
            None => Ok(repository.is_default_identity_enrolled().await?),
        }
    }

    #[instrument(skip_all)]
    pub async fn is_default_identity_enrolled(&self) -> Result<bool> {
        Ok(self
            .enrollment_repository()
            .is_default_identity_enrolled()
            .await?)
    }

    #[instrument(skip_all)]
    pub async fn identity_should_enroll(&self, name: &Option<String>, force: bool) -> Result<bool> {
        if force {
            return Ok(true);
        }

        // Force enrollment if there are no spaces or projects in the database
        if self.get_spaces().await?.is_empty() || self.projects().get_projects().await?.is_empty() {
            return Ok(true);
        }

        // Force enrollment if the identity is not enrolled
        Ok(!self.is_identity_enrolled(name).await?)
    }

    #[instrument(skip_all, fields(identifier = %identifier))]
    pub async fn set_identifier_as_enrolled(
        &self,
        identifier: &Identifier,
        email: &EmailAddress,
    ) -> Result<()> {
        Ok(self
            .enrollment_repository()
            .set_as_enrolled(identifier, email)
            .await?)
    }

    /// Return information of enrolled entities. Either:
    ///
    ///  - all the currently enrolled entities
    ///  - all the known identities and their corresponding enrollment state
    #[instrument(skip_all, fields(filter = %filter))]
    pub async fn get_identity_enrollments(
        &self,
        filter: EnrollmentFilter,
    ) -> Result<Vec<IdentityEnrollment>> {
        let repository = self.enrollment_repository();
        match filter {
            EnrollmentFilter::Enrolled => Ok(repository.get_enrolled_identities().await?),
            EnrollmentFilter::Any => Ok(repository.get_all_identities_enrollments().await?),
        }
    }

    /// Return enrollment information of the identity with the given name
    pub async fn get_identity_enrollment(&self, name: &str) -> Result<Option<IdentityEnrollment>> {
        let identifier = self.get_identifier_by_name(name).await?;
        let repository = self.enrollment_repository();
        Ok(repository
            .get_enrolled_identities()
            .await?
            .into_iter()
            .find(|e| e.identifier() == &identifier))
    }

    /// Return true if the user is enrolled.
    /// At the moment this check only verifies that there is a default project.
    /// This project should be the project that is created at the end of the enrollment procedure
    #[instrument(skip_all)]
    pub async fn is_enrolled(&self) -> miette::Result<bool> {
        if !self.is_default_identity_enrolled().await? {
            return Ok(false);
        }

        let default_space_exists = self.get_default_space().await.is_ok();
        if !default_space_exists {
            let message =
                "There should be a default space set for the current user. Please re-enroll";
            error!("{}", message);
            return Err(CliStateError::Other(message.into()))?;
        }

        let default_project_exists = self.projects().get_default_project().await.is_ok();
        if !default_project_exists {
            let message =
                "There should be a default project set for the current user. Please re-enroll";
            error!("{}", message);
            return Err(CliStateError::Other(message.into()))?;
        }

        Ok(true)
    }
}

#[derive(Debug)]
pub enum EnrollmentFilter {
    Enrolled,
    Any,
}

impl Display for EnrollmentFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EnrollmentFilter::Enrolled => f.write_str("enrolled"),
            EnrollmentFilter::Any => f.write_str("any"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum EnrollmentStatus {
    Enrolled {
        at: TimestampInSeconds,
        email: Option<EmailAddress>,
    },
    NotEnrolled,
}

impl EnrollmentStatus {
    pub fn is_enrolled(&self) -> bool {
        matches!(self, EnrollmentStatus::Enrolled { .. })
    }

    pub fn email(&self) -> Option<&EmailAddress> {
        match self {
            EnrollmentStatus::Enrolled { email, .. } => email.as_ref(),
            EnrollmentStatus::NotEnrolled => None,
        }
    }
}

#[derive(Serialize)]
pub struct IdentityEnrollment {
    identifier: Identifier,
    name: String,
    is_default: bool,
    status: EnrollmentStatus,
}

impl IdentityEnrollment {
    pub fn new(
        identifier: Identifier,
        name: String,
        is_default: bool,
        enrolled_at: Option<OffsetDateTime>,
        email: Option<EmailAddress>,
    ) -> Self {
        let status = match enrolled_at {
            Some(enrolled_at) => EnrollmentStatus::Enrolled {
                at: TimestampInSeconds::from(enrolled_at.unix_timestamp() as u64),
                email,
            },
            None => EnrollmentStatus::NotEnrolled,
        };
        Self {
            identifier,
            name,
            is_default,
            status,
        }
    }
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn status(&self) -> &EnrollmentStatus {
        &self.status
    }
}

impl Display for IdentityEnrollment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", fmt::PADDING, color_primary(self.name()))?;
        if self.is_default {
            write!(f, " (default)")?;
        }
        writeln!(f, ":")?;

        writeln!(
            f,
            "{}{}With Identifier {}",
            fmt::PADDING,
            fmt::INDENTATION,
            color_primary(self.identifier().to_string())
        )?;

        match &self.status {
            EnrollmentStatus::Enrolled { at, email } => {
                write!(
                    f,
                    "{}{}Was {} at {}",
                    fmt::PADDING,
                    fmt::INDENTATION,
                    color_ok("enrolled"),
                    color_primary(human_readable_time(*at))
                )?;
                if let Some(email) = email {
                    writeln!(f, " with email {}", color_primary(email.to_string()))?;
                } else {
                    writeln!(f)?;
                }
            }
            EnrollmentStatus::NotEnrolled => {
                writeln!(
                    f,
                    "{}{}Is {}",
                    fmt::PADDING,
                    fmt::INDENTATION,
                    color_warn("not enrolled")
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LegacyEnrollmentTicket {
    one_time_code: OneTimeCode,
    project: ProjectModel,
}

impl LegacyEnrollmentTicket {
    fn new(one_time_code: OneTimeCode, project: ProjectModel) -> Self {
        Self {
            one_time_code,
            project,
        }
    }

    pub fn hex_encoded(&self) -> Result<String> {
        let serialized = serde_json::to_vec(&self)
            .map_err(|_err| ApiError::core("Failed to hex-encode enrollment ticket"))?;
        Ok(hex::encode(serialized))
    }
}

impl FromStr for LegacyEnrollmentTicket {
    type Err = ApiError;

    fn from_str(contents: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok(data) = hex::decode(contents) {
            debug!(%contents, "decoding hex-encoded LegacyEnrollmentTicket");
            Ok(serde_json::from_slice(&data).map_err(|_err| {
                ApiError::core(
                    "Failed to decode LegacyEnrollmentTicket json from hex-encoded string",
                )
            })?)
        } else {
            debug!(%contents, "decoding LegacyEnrollmentTicket from raw contents");
            Ok(serde_json::from_str(contents).map_err(|_err| {
                ApiError::core("Failed to decode LegacyEnrollmentTicket json from raw contents")
            })?)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExportedEnrollmentTicket {
    pub one_time_code: OneTimeCode,
    project_route: ProjectRoute,
    project_identifier: Identifier,
    project_name: String,
    project_change_history: String,
    authority_change_history: String,
    authority_route: MultiAddr,
}

impl ExportedEnrollmentTicket {
    const MANDATORY_FIELDS_NUM: usize = 7;

    pub fn new(
        one_time_code: OneTimeCode,
        project_route: ProjectRoute,
        project_identifier: Identifier,
        project_name: impl Into<String>,
        project_change_history: impl Into<String>,
        authority_change_history: impl Into<String>,
        authority_route: MultiAddr,
    ) -> Self {
        Self {
            one_time_code,
            project_route,
            project_identifier,
            project_name: project_name.into(),
            project_change_history: project_change_history.into(),
            authority_change_history: authority_change_history.into(),
            authority_route,
        }
    }

    #[cfg(any(test, feature = "test-utils"))]
    pub fn new_test() -> Self {
        Self::new(
            OneTimeCode::new(),
            ProjectRoute::new(MultiAddr::from_str("/dnsaddr/project_id.projects.orchestrator.ockam.io/tcp/443/service/project_id/service/api").unwrap()).unwrap(),
            Identifier::from_str("I5cf1bc8d300018d9a0fa6a177c073347abe35f95e55837b23e22a5f6857a1e0c").unwrap(),
            crate::cli_state::random_name(),
            "81825837830101583285f68200815820245ba33c7729dce1c94d8c1a00fcf89a7af33689d4563176f9dffbdd147d4488f41a66e2ee7b1a79aef17b820081584070856bb8da621154a39c894a2fedded55257715b00940b9cffe54b51d87889aff2c077124ee6e0e1c2e711688470affbc65d909c87acf4e41d38bdfb03e2000d",
            "81825837830101583285f6820081582045d9dac79f226762025fc82e7407aee4a4c8e7068dc04edd44f1c777b8f0cf6bf41a66e2ee7b1a79aef17b8200815840c65ce655fd57cf2ea0b0679066a24bc99e2b223341186b5eaec951101f291e96c5fc8343291a23cbd8dc063ad1f9a9554f036e8f34ab5388e444977e7e29ab0b",
            MultiAddr::from_str("/dnsaddr/project_id.projects.orchestrator.ockam.io/tcp/443/service/project_id/service/authority/service/api").unwrap(),
        )
    }

    pub async fn import(self) -> Result<EnrollmentTicket> {
        EnrollmentTicket::new(
            self.one_time_code,
            self.project_route.id,
            self.project_name,
            self.project_route.route,
            self.project_change_history,
            self.authority_change_history,
            self.authority_route,
        )
        .await
    }

    pub fn hex_encoded(&self) -> Result<String> {
        Ok(hex::encode(self.to_string()))
    }
}

impl FromStr for ExportedEnrollmentTicket {
    type Err = ApiError;

    fn from_str(contents: &str) -> std::result::Result<Self, Self::Err> {
        // Try to decode from hex-encoded string
        let contents = match hex::decode(contents) {
            Ok(decoded) => String::from_utf8(decoded)
                .map_err(|_| ApiError::core("Failed to hex decode enrollment ticket"))?,
            Err(_) => contents.to_string(),
        };

        // Try to decode from json
        let contents = match serde_json::from_str(&contents) {
            Ok(decoded) => return Ok(decoded),
            Err(_) => contents,
        };

        // Decode as comma-separated text
        let values: Vec<&str> = contents.split(',').collect();
        if values.len() < Self::MANDATORY_FIELDS_NUM {
            error!(%contents, ?values, "missing fields in enrollment ticket: expected at least {}, got {}", Self::MANDATORY_FIELDS_NUM, values.len());
            return Err(ApiError::core("Missing fields in enrollment ticket").into());
        }
        let (
            project_route,
            project_identifier,
            project_name,
            one_time_code,
            project_change_history,
            authority_change_history,
            authority_route,
        ) = (
            values[0], values[1], values[2], values[3], values[4], values[5], values[6],
        );

        Ok(Self::new(
            OneTimeCode::from_str(one_time_code)?,
            ProjectRoute::new(MultiAddr::from_str(project_route)?)?,
            Identifier::from_str(project_identifier)?,
            project_name.to_string(),
            project_change_history.to_string(),
            authority_change_history.to_string(),
            MultiAddr::from_str(authority_route)?,
        ))
    }
}

impl Display for ExportedEnrollmentTicket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{},{}",
            self.project_route.route,
            self.project_identifier,
            self.project_name,
            String::from(&self.one_time_code),
            self.project_change_history,
            self.authority_change_history,
            self.authority_route,
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProjectRoute {
    id: String,
    route: MultiAddr,
}

impl ProjectRoute {
    pub fn new(route: MultiAddr) -> Result<Self> {
        // from a project route like:
        // "/dnsaddr/<hostname>/tcp/443/service/<id>/service/api"
        // extract the "<id>" segment
        let id = route
            .iter()
            .find_map(|pv| {
                if pv.code() == Service::CODE {
                    String::from_utf8(pv.data().to_vec()).ok()
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                ApiError::core(format!(
                    "Couldn't extract the project id from route {route}"
                ))
            })?;
        Ok(Self { id, route })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnrollmentTicket {
    pub one_time_code: OneTimeCode,
    project_id: String,
    project_name: String,
    project_route: MultiAddr,
    project_identity: Identity,
    authority_identity: Identity,
    authority_route: MultiAddr,
}

impl EnrollmentTicket {
    pub async fn new(
        one_time_code: OneTimeCode,
        project_id: impl Into<String>,
        project_name: impl Into<String>,
        project_route: MultiAddr,
        project_change_history: impl Into<String>,
        authority_change_history: impl Into<String>,
        authority_route: MultiAddr,
    ) -> Result<Self> {
        debug!("Creating enrollment ticket");
        let project_id = project_id.into();
        let project_change_history = project_change_history.into();
        let project_identity = Identity::import_from_string(
            None,
            &project_change_history,
            Vault::create_verifying_vault(),
        )
        .await?;
        let authority_change_history = authority_change_history.into();
        let authority_identity = Identity::import_from_string(
            None,
            &authority_change_history,
            Vault::create_verifying_vault(),
        )
        .await?;
        Ok(Self {
            one_time_code,
            project_id,
            project_name: project_name.into(),
            project_route,
            project_identity,
            authority_identity,
            authority_route,
        })
    }

    pub async fn new_from_project(
        one_time_code: OneTimeCode,
        project: &ProjectModel,
    ) -> Result<Self> {
        debug!(?project, "Creating enrollment ticket from project");
        let project_change_history = project
            .project_change_history
            .as_ref()
            .ok_or_else(|| ApiError::core("no project change history"))?;
        let authority_change_history = project
            .authority_identity
            .as_ref()
            .ok_or_else(|| ApiError::core("no authority change history"))?;
        let authority_route = project
            .authority_access_route
            .as_ref()
            .ok_or_else(|| ApiError::core("no authority route"))?;
        Self::new(
            one_time_code,
            &project.id,
            &project.name,
            MultiAddr::from_str(&project.access_route)?,
            project_change_history,
            authority_change_history,
            MultiAddr::from_str(authority_route)?,
        )
        .await
    }

    pub async fn new_from_legacy(ticket: LegacyEnrollmentTicket) -> Result<Self> {
        debug!(?ticket, "Creating enrollment ticket from legacy ticket");
        let project = ticket.project;
        let project_id = project.id.clone();
        let project_name = project.name.clone();
        let project_change_history = project
            .project_change_history
            .as_ref()
            .ok_or_else(|| ApiError::core("no project change history in legacy ticket"))?;
        let authority_change_history = project
            .authority_identity
            .as_ref()
            .ok_or_else(|| ApiError::core("no authority change history in legacy ticket"))?;
        let authority_route = project
            .authority_access_route
            .as_ref()
            .ok_or_else(|| ApiError::core("no authority route in legacy ticket"))?;
        Self::new(
            ticket.one_time_code,
            project_id,
            project_name,
            MultiAddr::from_str(&project.access_route)?,
            project_change_history,
            authority_change_history,
            MultiAddr::from_str(authority_route)?,
        )
        .await
    }

    pub fn project(&self) -> Result<ProjectModel> {
        Ok(ProjectModel {
            id: self.project_id.clone(),
            name: self.project_name.clone(),
            space_name: "".to_string(),
            access_route: self.project_route.to_string(),
            users: vec![],
            space_id: "".to_string(),
            identity: Some(self.project_identity.identifier().clone()),
            authority_access_route: Some(self.authority_route.to_string()),
            authority_identity: Some(self.authority_identity.export_as_string()?),
            okta_config: None,
            kafka_config: None,
            version: None,
            running: None,
            operation_id: None,
            user_roles: vec![],
            project_change_history: Some(self.project_identity.export_as_string()?),
        })
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn set_project_name(&mut self, name: impl Into<String>) {
        self.project_name = name.into();
    }

    pub fn export(self) -> Result<ExportedEnrollmentTicket> {
        Ok(ExportedEnrollmentTicket::new(
            self.one_time_code,
            ProjectRoute::new(self.project_route)?,
            self.project_identity.identifier().clone(),
            self.project_name,
            self.project_identity.export_as_string()?,
            self.authority_identity.export_as_string()?,
            self.authority_route,
        ))
    }

    pub fn export_legacy(self) -> Result<LegacyEnrollmentTicket> {
        let project = self.project()?;
        Ok(LegacyEnrollmentTicket::new(self.one_time_code, project))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn text_export_as_legacy() {
        let ticket = ExportedEnrollmentTicket::new_test();
        let exported = ticket.import().await.unwrap();
        let legacy = exported.clone().export_legacy().unwrap();
        assert_eq!(legacy.one_time_code, exported.one_time_code);
        assert_eq!(legacy.project, exported.project().unwrap());
    }

    #[test]
    fn test_enrollment_ticket_encoding_decoding() {
        let exported = ExportedEnrollmentTicket::new_test();
        let plain = exported.to_string();
        assert!(plain.contains(&String::from(&exported.one_time_code)));
        assert!(plain.contains(&exported.project_route.id));
        assert!(plain.contains(&exported.project_route.route.to_string()));
        assert!(plain.contains(&exported.project_name));
        assert!(plain.contains(&exported.project_change_history));
        assert!(plain.contains(&exported.authority_change_history));

        let decoded = ExportedEnrollmentTicket::from_str(&plain).unwrap();
        assert_eq!(decoded, exported);

        let json_encoded = serde_json::to_string(&exported).unwrap();
        let decoded = ExportedEnrollmentTicket::from_str(&json_encoded).unwrap();
        assert_eq!(decoded, exported);

        let encoded = exported.hex_encoded().unwrap();
        let decoded = ExportedEnrollmentTicket::from_str(&encoded).unwrap();
        assert_eq!(decoded, exported);
    }

    #[test]
    fn test_project_route() {
        let project_id = "c4a6a4b4-537b-4f2e-ace4-ef1992b922a6";
        let route = MultiAddr::from_str(&format!("/dnsaddr/{project_id}.projects.orchestrator.ockam.io/tcp/443/service/{project_id}/service/api")).unwrap();
        let from_route = ProjectRoute::new(route.clone()).unwrap();
        assert_eq!(from_route.id, project_id);
        assert_eq!(from_route.route, route);

        let route = MultiAddr::from_str(&format!(
            "/dnsaddr/hostname.io/tcp/443/service/{project_id}/service/api"
        ))
        .unwrap();
        let from_route = ProjectRoute::new(route.clone()).unwrap();
        assert_eq!(from_route.id, project_id);
        assert_eq!(from_route.route, route);

        let from_invalid_route = ProjectRoute::new(MultiAddr::from_str("/node/n1").unwrap());
        assert!(from_invalid_route.is_err());
    }

    #[tokio::test]
    async fn test_enrollment_ticket_from_legacy() {
        let otc = OneTimeCode::new();
        let project_id = "c4a6a4b4-537b-4f2e-ace4-ef1992b922a6";
        let project_name = "name";
        let project_change_history = "81825837830101583285f68200815820245ba33c7729dce1c94d8c1a00fcf89a7af33689d4563176f9dffbdd147d4488f41a66e2ee7b1a79aef17b820081584070856bb8da621154a39c894a2fedded55257715b00940b9cffe54b51d87889aff2c077124ee6e0e1c2e711688470affbc65d909c87acf4e41d38bdfb03e2000d";
        let authority_change_history = "81825837830101583285f6820081582045d9dac79f226762025fc82e7407aee4a4c8e7068dc04edd44f1c777b8f0cf6bf41a66e2ee7b1a79aef17b8200815840c65ce655fd57cf2ea0b0679066a24bc99e2b223341186b5eaec951101f291e96c5fc8343291a23cbd8dc063ad1f9a9554f036e8f34ab5388e444977e7e29ab0b";
        let project = ProjectModel {
            id: project_id.to_string(),
            name: project_name.to_string(),
            space_name: "".to_string(),
            access_route: "/dnsaddr/project.ockam.io/tcp/443".to_string(),
            users: vec![],
            space_id: "".to_string(),
            identity: None,
            authority_access_route: Some("/dnsaddr/authority.ockam.io/tcp/443".to_string()),
            authority_identity: Some(authority_change_history.to_string()),
            okta_config: None,
            kafka_config: None,
            version: None,
            running: None,
            operation_id: None,
            user_roles: vec![],
            project_change_history: Some(project_change_history.to_string()),
        };
        let legacy = LegacyEnrollmentTicket::new(otc, project.clone());
        let enrollment_ticket = EnrollmentTicket::new_from_legacy(legacy).await.unwrap();
        assert_eq!(enrollment_ticket.one_time_code, otc);
        assert_eq!(enrollment_ticket.project_id, project_id);
        assert_eq!(enrollment_ticket.project_name, project_name);
        assert_eq!(
            &enrollment_ticket.project_identity,
            &Identity::import_from_string(
                None,
                project_change_history,
                Vault::create_verifying_vault()
            )
            .await
            .unwrap()
        );
        assert_eq!(
            &enrollment_ticket.project_route,
            &MultiAddr::from_str("/dnsaddr/project.ockam.io/tcp/443").unwrap()
        );
        assert_eq!(
            &enrollment_ticket.authority_identity,
            &Identity::import_from_string(
                None,
                authority_change_history,
                Vault::create_verifying_vault()
            )
            .await
            .unwrap()
        );
        assert_eq!(
            &enrollment_ticket.authority_route,
            &MultiAddr::from_str("/dnsaddr/authority.ockam.io/tcp/443").unwrap()
        );
    }

    #[tokio::test]
    async fn test_enrollment_ticket_from_exported() {
        let otc = OneTimeCode::new();
        let project_id = "c4a6a4b4-537b-4f2e-ace4-ef1992b922a6";
        let project_route = MultiAddr::from_str("/dnsaddr/c4a6a4b4-537b-4f2e-ace4-ef1992b922a6.projects.orchestrator.ockam.io/tcp/443/service/c4a6a4b4-537b-4f2e-ace4-ef1992b922a6/service/api").unwrap();
        let project_name = "name";
        let project_change_history = "81825837830101583285f68200815820245ba33c7729dce1c94d8c1a00fcf89a7af33689d4563176f9dffbdd147d4488f41a66e2ee7b1a79aef17b820081584070856bb8da621154a39c894a2fedded55257715b00940b9cffe54b51d87889aff2c077124ee6e0e1c2e711688470affbc65d909c87acf4e41d38bdfb03e2000d";
        let project_identity = Identity::import_from_string(
            None,
            project_change_history,
            Vault::create_verifying_vault(),
        )
        .await
        .unwrap();
        let authority_change_history = "81825837830101583285f6820081582045d9dac79f226762025fc82e7407aee4a4c8e7068dc04edd44f1c777b8f0cf6bf41a66e2ee7b1a79aef17b8200815840c65ce655fd57cf2ea0b0679066a24bc99e2b223341186b5eaec951101f291e96c5fc8343291a23cbd8dc063ad1f9a9554f036e8f34ab5388e444977e7e29ab0b";
        let authority_identity = Identity::import_from_string(
            None,
            authority_change_history,
            Vault::create_verifying_vault(),
        )
        .await
        .unwrap();
        let authority_route = "/dnsaddr/c4a6a4b4-537b-4f2e-ace4-ef1992b922a6.projects.orchestrator.ockam.io/tcp/443/service/c4a6a4b4-537b-4f2e-ace4-ef1992b922a6/service/authority/service/api";
        let exported = ExportedEnrollmentTicket::new(
            otc,
            ProjectRoute::new(project_route.clone()).unwrap(),
            project_identity.identifier().clone(),
            project_name,
            project_change_history,
            authority_change_history,
            MultiAddr::from_str(authority_route).unwrap(),
        );
        let enrollment_ticket = exported.clone().import().await.unwrap();
        assert_eq!(enrollment_ticket.project_id, project_id);
        assert_eq!(enrollment_ticket.project_route, project_route);
        assert_eq!(enrollment_ticket.project_name, project_name);
        assert_eq!(&enrollment_ticket.project_identity, &project_identity);
        assert_eq!(&enrollment_ticket.authority_identity, &authority_identity);
        assert_eq!(&enrollment_ticket.one_time_code, &otc);

        let exported_back = enrollment_ticket.clone().export().unwrap();
        assert_eq!(exported_back.project_route, exported.project_route);
        assert_eq!(
            exported_back.project_identifier,
            exported.project_identifier
        );
        assert_eq!(exported_back.project_name, exported.project_name);
        assert_eq!(exported_back.one_time_code, exported.one_time_code);
        assert_eq!(
            exported_back.project_change_history,
            exported.project_change_history
        );
        assert_eq!(
            exported_back.authority_change_history,
            exported.authority_change_history
        );
        assert_eq!(exported_back.authority_route, exported.authority_route);
    }
}
