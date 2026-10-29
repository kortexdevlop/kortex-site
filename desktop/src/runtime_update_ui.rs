use crate::{api::Api, config::Config};
use dioxus::prelude::*;
#[component]
pub fn RuntimeUpdate(config: Config) -> Element {
    let version = env!("CARGO_PKG_VERSION");
    let mut busy = use_signal(|| false);
    let mut notice = use_signal(String::new);
    let mut candidate = use_signal(|| None::<kortex_verify::runtime::Desktop>);
    rsx! {section{class:"tool-card",h2{"Kortex Desktop"}p{"Version {version}"}
    button{disabled:*busy.read(),onclick:move |_|{let c=config.clone();busy.set(true);candidate.set(None);spawn(async move{let result=tokio::task::spawn_blocking(move||{let api=Api::new(&c).map_err(|e|e.to_string())?;crate::runtime_update::check(&c,&api)}).await;match result{Ok(Ok(Some(d)))=>{notice.set(format!("Kortex {} est disponible. Le téléchargement ne remplace pas votre installation automatiquement.",d.version));candidate.set(Some(d));},Ok(Ok(None))=>notice.set("Aucune nouvelle version de Kortex Desktop disponible.".into()),Ok(Err(e))=>notice.set(e),Err(_)=>notice.set("Vérification de Kortex indisponible.".into())}busy.set(false);});},"Vérifier la version de Kortex"}
    p{role:"status","{notice}"}if let Some(d)=candidate.read().clone(){if !d.code_signed{p{"Version de test staging non signée pour Windows."}}a{href:d.url,rel:"noreferrer noopener",target:"_blank","Télécharger cette version"}details{summary{"Vérifier le fichier téléchargé"}code{"SHA-256 : {d.sha256}"}}}
    }}
}
