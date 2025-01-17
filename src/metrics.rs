use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

mod export_to_sheets;
mod gql;
mod list_repos;
mod print;
mod repo_participants;
mod util;

#[async_trait]
pub trait Producer {
    fn column_names(&self) -> Vec<String>;
    async fn producer_task(self, tx: Sender<Vec<String>>) -> anyhow::Result<()>;
}

#[async_trait]
pub trait Consumer {
    async fn consume(
        self,
        rx: &mut Receiver<Vec<String>>,
        column_names: Vec<String>,
    ) -> anyhow::Result<()>;
}

pub use export_to_sheets::ExportToSheets;
pub use gql::Graphql;
pub use list_repos::ListReposForOrg;
pub use print::Print;
pub use repo_participants::RepoParticipants;
