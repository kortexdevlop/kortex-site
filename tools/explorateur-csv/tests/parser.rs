use tool_explorateur_csv::{parser::{parse,Parser,MAX_COLUMNS,MAX_ROWS},stats::{Analyzer,summarize,parse_number,report_csv,Kind},summarize_csv,export_report};
use serde_json::json;
#[test]fn quotes_newlines_bom_and_crlf(){
 let t=parse("\u{feff}nom,note\r\n\"Dupont, Jean\",12\r\n\"a \"\"cité\"\"\nligne\",13.5\r\n".as_bytes()).unwrap();
 assert_eq!(t.headers,["nom","note"]);assert_eq!(t.row(0),["Dupont, Jean","12"]);assert_eq!(t.row(1),["a \"cité\"\nligne","13.5"]);
 assert_eq!(parse(b"a\nb").unwrap().row(0),["b"]);assert_eq!(parse(b"a,b\n,\n").unwrap().row(0),["",""]);
}
#[test]fn hostile_inputs_are_refused_with_explicit_codes(){
 for (input,code) in [(&b"a,b\n1,\"2\n"[..],"UNTERMINATED_QUOTE"),(&b"a,b\n1,x\"y\n"[..],"MALFORMED_QUOTE"),(&b"a,b\n\"1\"x,2\n"[..],"MALFORMED_QUOTE"),(&b"a,b\n1\n"[..],"RAGGED_ROW"),(&b"a,b\n1,2,3\n"[..],"RAGGED_ROW"),
  (&b"a,b\n\xff,2\n"[..],"INVALID_UTF8"),(&b"a,b\n\xc3\x28,2\n"[..],"INVALID_UTF8"),(&b"a\n1\r2\n"[..],"INVALID_LINE_ENDING"),(&b"a\n\x00\n"[..],"INVALID_CHARACTER"),(&b""[..],"EMPTY"),(&b" \n \n"[..],"EMPTY")]{
  assert_eq!(parse(input).unwrap_err(),code,"{:?}",String::from_utf8_lossy(input));
 }
}
#[test]fn bounds_are_exact(){
 let wide=|n:usize|(0..n).map(|i|format!("c{i}")).collect::<Vec<_>>().join(",");
 assert!(parse(wide(MAX_COLUMNS).as_bytes()).is_ok());assert_eq!(parse(wide(MAX_COLUMNS+1).as_bytes()).unwrap_err(),"TOO_LARGE");
 let long=|n:usize|format!("a\n{}","1\n".repeat(n));assert_eq!(parse(long(MAX_ROWS).as_bytes()).unwrap().rows(),MAX_ROWS);assert_eq!(parse(long(MAX_ROWS+1).as_bytes()).unwrap_err(),"TOO_LARGE");
 assert_eq!(parse(&vec![b'a';10*1024*1024+1]).unwrap_err(),"TOO_LARGE");
}
#[test]fn numbers_are_strict_decimal_and_never_nan_or_infinite(){
 for ok in ["0","-0","+1.5","1e3","1E-3",".5","5.","  42  "]{assert!(parse_number(ok).unwrap().is_some(),"{ok}");}
 for text in ["NaN","inf","Infinity","-inf","0x10","1_000","1,5","e5","1e","--1","","abc","1 2"]{assert_eq!(parse_number(text).unwrap(),None,"{text}");}
 assert_eq!(parse_number("1e999").unwrap_err(),"NON_FINITE_NUMBER");assert_eq!(parse_number("-0").unwrap().unwrap().to_string(),"0");
 let t=parse(b"v\n1e999\n").unwrap();assert_eq!(summarize(&t).unwrap_err(),"NON_FINITE_NUMBER");
 let t=parse(b"v\n1e308\n1e308\n").unwrap();assert_eq!(summarize(&t).unwrap_err(),"NUMERIC_OVERFLOW");
}
#[test]fn golden_statistics_and_null_policy(){
 let t=parse("ville,age,score,note\nParis,30,1.5,\nLyon,,2.5,x\nParis, ,-4,y\nNice,10,,\n".as_bytes()).unwrap();let s=summarize(&t).unwrap();
 assert_eq!((s[0].kind,s[0].count,s[0].nulls),(Kind::Text,4,0));assert_eq!((s[0].min,s[0].max,s[0].mean),(None,None,None));
 assert_eq!((s[1].kind,s[1].count,s[1].nulls,s[1].min,s[1].max,s[1].mean),(Kind::Number,2,2,Some(10.0),Some(30.0),Some(20.0)));
 assert_eq!((s[2].count,s[2].nulls,s[2].min,s[2].max,s[2].mean),(3,1,Some(-4.0),Some(2.5),Some(0.0)));
 assert_eq!((s[3].kind,s[3].count,s[3].nulls),(Kind::Text,2,2));
 let all_null=summarize(&parse(b"a\n\n \n").unwrap()).unwrap();assert_eq!((all_null[0].kind,all_null[0].count,all_null[0].nulls),(Kind::Text,0,2));
}
#[test]fn actions_return_json_without_non_finite_numbers_and_share_the_ui_parser(){
 let out=summarize_csv(&json!({"data_text":"a,b\n1,x\n3,\n"})).unwrap();
 assert_eq!(out,json!({"rows":2,"columns":2,"stats":[{"name":"a","kind":"number","count":2,"nulls":0,"min":1.0,"max":3.0,"mean":2.0},{"name":"b","kind":"text","count":1,"nulls":1}]}));
 assert!(!out.to_string().contains(":null")&&!out.to_string().contains("NaN"));
 for bad in [json!({}),json!({"data_text":1}),json!({"data_text":"a","x":1}),json!({"data_text":"a".repeat(1_000_001)}),json!({"data_text":"a\n\"1"})]{assert!(summarize_csv(&bad).is_err());assert!(export_report(&bad).is_err());}
 assert_eq!(export_report(&json!({"data_text":"a,b\n1,x\n3,\n"})).unwrap()["report_csv"],"colonne,type,valeurs,vides,min,max,moyenne\na,nombre,2,0,1,3,2\nb,texte,1,1,,,\n");
}
#[test]fn exported_report_neutralises_formulas_and_quotes_names(){
 let t=parse("=cmd|' /C calc'!A0,\"x,y\"\n1,2\n".as_bytes()).unwrap();let report=report_csv(&summarize(&t).unwrap());
 assert!(report.lines().nth(1).unwrap().starts_with("'=cmd|' /C calc'!A0,nombre"));assert!(report.contains("\"x,y\",nombre"));
}
#[test]fn streaming_is_independent_of_chunk_boundaries_including_utf8_crlf_and_quotes(){
 let text="\u{feff}nom,café\r\n\"é, \"\"x\"\"\nz\",12.5\r\nnaïve ☃,\r\n,-3\r\n";let whole=parse(text.as_bytes()).unwrap();
 for size in [1,2,3,5,7,64]{let mut parser=Parser::new();for chunk in text.as_bytes().chunks(size){parser.feed(chunk).unwrap();}let table=parser.finish().unwrap();assert_eq!(table,whole,"chunk size {size}");}
 let whole_stats={let mut a=Analyzer::new();a.feed(text.as_bytes()).unwrap();a.finish().unwrap().1};
 for size in [1,3,11]{let mut a=Analyzer::new();for chunk in text.as_bytes().chunks(size){a.feed(chunk).unwrap();}assert_eq!(a.finish().unwrap().1,whole_stats);}
 let mut parser=Parser::new();parser.feed(b"a\r").unwrap();assert_eq!(parser.feed(b"b").unwrap_err(),"INVALID_LINE_ENDING");
 let mut parser=Parser::new();parser.feed(&"é".as_bytes()[..1]).unwrap();assert_eq!(parser.finish().unwrap_err(),"INVALID_UTF8");
 let mut parser=Parser::new();parser.feed(b"a\n\xc2").unwrap();assert_eq!(parser.feed(b"\x85").unwrap_err(),"INVALID_CHARACTER");
}
#[test]fn fast_number_path_matches_the_standard_parser(){
 for text in ["0.1","0.30000000000000004","123456789012345","1234567.891","-0.000001","4.9e-324","1.7976931348623157e308","100000000000000000000","3.141592653589793","0.1e1","007.50"]{let ours=parse_number(text).unwrap().unwrap();assert_eq!(ours.to_bits(),text.parse::<f64>().unwrap().to_bits(),"{text}");}
}
