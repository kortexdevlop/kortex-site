use crate::{PdfTool, explain};
use serde_json::{Value, json};
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.into()
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}
fn kilobytes(bytes: usize) -> String {
    format!("{} Ko", bytes.div_ceil(1000))
}
impl PdfTool {
    pub(crate) fn render(&self) -> Value {
        let busy = self.host.pending_count() > 0;
        let loading = ["picking", "reading"].contains(&self.phase);
        let ready = self.slots.iter().all(Option::is_some);
        let message = match (self.phase, &self.merged) {
            ("picking", _) => "Choisissez un fichier PDF après autorisation.".to_string(),
            ("reading", _) => format!("Lecture locale : {} / {} octets", self.offset, self.size),
            ("merged", Some((bytes, pages))) => format!(
                "Fusion terminée : {pages} pages, {}. Aucun fichier envoyé.",
                kilobytes(bytes.len())
            ),
            ("cancelled", _) => {
                "Opération annulée. Les fichiers déjà chargés sont conservés.".into()
            }
            ("error", _) => format!("{} ({})", explain(&self.error), self.error),
            _ => "Choisissez deux PDF non protégés de 700 Ko maximum chacun.".into(),
        };
        let rows: Vec<Value> = self
            .slots
            .iter()
            .enumerate()
            .map(|(i, slot)| match slot {
                Some(s) => json!({"rank":(i+1).to_string(),"name":clip(&s.name,100),"pages":s.pages,"size":kilobytes(s.data.len())}),
                None => json!({"rank":(i+1).to_string(),"name":"Aucun fichier","pages":"","size":""}),
            })
            .collect();
        let progress = if self.phase == "merged" {
            100.0
        } else if self.size == 0 {
            0.0
        } else {
            self.offset as f64 / self.size as f64 * 100.0
        };
        let mut children = vec![
            node(
                "heading",
                "title",
                json!({"text":"Fusion PDF","level":1}),
                vec![],
            ),
            node(
                "text",
                "intro",
                json!({"text":"Les pages du premier PDF sont suivies de celles du second. Signets, formulaires et destinations nommées ne sont pas conservés."}),
                vec![],
            ),
            node(
                "row",
                "pickers",
                json!({"gap":8}),
                vec![
                    node(
                        "button",
                        "pick_first",
                        json!({"text":"Choisir le premier PDF","disabled":busy||loading}),
                        vec![],
                    ),
                    node(
                        "button",
                        "pick_second",
                        json!({"text":"Choisir le second PDF","disabled":busy||loading}),
                        vec![],
                    ),
                    node(
                        "button",
                        "cancel",
                        json!({"text":"Annuler","disabled":!loading}),
                        vec![],
                    ),
                ],
            ),
            node(
                "table",
                "files",
                json!({"columns":[{"key":"rank","label":"Ordre"},{"key":"name","label":"Fichier"},{"key":"pages","label":"Pages"},{"key":"size","label":"Taille"}],"rows":rows}),
                vec![],
            ),
            node(
                "progress",
                "progress",
                json!({"label":"Progression","value":progress}),
                vec![],
            ),
            node(
                "alert",
                "status",
                json!({"text":message,"severity":if self.phase=="error"{"error"}else{"info"}}),
                vec![],
            ),
            node(
                "row",
                "actions",
                json!({"gap":8}),
                vec![
                    node(
                        "button",
                        "swap",
                        json!({"text":"Inverser l’ordre","disabled":busy||loading||!ready}),
                        vec![],
                    ),
                    node(
                        "button",
                        "merge",
                        json!({"text":"Fusionner","disabled":busy||loading||!ready}),
                        vec![],
                    ),
                    node(
                        "button",
                        "save",
                        json!({"text":"Enregistrer le PDF fusionné","disabled":busy||self.merged.is_none()}),
                        vec![],
                    ),
                    node(
                        "button",
                        "reset",
                        json!({"text":"Recommencer","disabled":busy&&!loading}),
                        vec![],
                    ),
                ],
            ),
        ];
        if !self.notice.is_empty() {
            children.push(node(
                "alert",
                "notice",
                json!({"text":clip(&self.notice,500),"severity":"info"}),
                vec![],
            ));
        }
        node("column", "pdf-tool", json!({"gap":12}), children)
    }
}
