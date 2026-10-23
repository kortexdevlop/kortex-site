use dioxus::prelude::*;
use serde_json::Value;
#[component]
pub(crate) fn Chart(data: Value) -> Element {
    let labels = data["labels"].as_array().unwrap();
    let series = data["series"].as_array().unwrap();
    let mut low = 0.0f64;
    let mut high = 0.0f64;
    for s in series {
        for v in s["values"].as_array().unwrap() {
            let n = v.as_f64().unwrap();
            low = low.min(n);
            high = high.max(n);
        }
    }
    let range = if high == low { 1.0 } else { high - low };
    let y = |v: f64| 180.0 - (v - low) / range * 160.0;
    let step = 560.0 / labels.len().max(1) as f64;
    let baseline = y(0.0);
    let mut bars = Vec::new();
    let mut lines = Vec::new();
    for (si, s) in series.iter().enumerate() {
        let mut points = Vec::new();
        for (i, v) in s["values"].as_array().unwrap().iter().enumerate() {
            let x = 20.0 + (i as f64 + 0.5) * step;
            let vy = y(v.as_f64().unwrap());
            points.push(format!("{x:.3},{vy:.3}"));
            let width = step * 0.8 / series.len().max(1) as f64;
            bars.push((
                x - step * 0.4 + si as f64 * width,
                vy.min(baseline),
                width,
                (vy - baseline).abs(),
                si,
            ));
        }
        lines.push((points.join(" "), si));
    }
    rsx! { figure { class:"kortex-chart",
        figcaption { "Graphique — " for s in series { span { {s["label"].as_str().unwrap()} " " } } }
        svg {view_box:"0 0 600 220",role:"img","aria-label":"Graphique : données disponibles dans le tableau", "focusable":"false",
            line { x1:"20",x2:"580",y1:"{baseline}",y2:"{baseline}",stroke:"currentColor" }
            if data["kind"]=="bar" { for (x,y,w,h,si) in bars { rect { x:"{x:.3}",y:"{y:.3}",width:"{w:.3}",height:"{h:.3}",class:"kortex-series s{si}" } } }
            else { for (points,si) in lines { polyline {points,fill:"none",stroke_width:"2",class:"kortex-series s{si}"} } }
            for (i,label) in labels.iter().enumerate() { text {x:"{20.0+(i as f64+0.5)*step}",y:"210",text_anchor:"middle","{label.as_str().unwrap()}"} }
        }
        details { summary {"Données du graphique"} table {
            caption {"Valeurs du graphique"}
            thead {tr {th {scope:"col","Catégorie"} for s in series {th {scope:"col",{s["label"].as_str().unwrap()}}}}}
            tbody {for (i,label) in labels.iter().enumerate() {tr {th {scope:"row","{label.as_str().unwrap()}"} for s in series {td {{s["values"][i].to_string()}}}}}}
        } }
    } }
}
