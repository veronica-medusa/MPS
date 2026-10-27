use crate::ast::*;
use std::collections::HashMap;

pub struct MailMergeEngine;

impl MailMergeEngine {
    /// Binds records to document template fields and produces an individualized document per record
    pub fn merge_records(template: &Document, records: &[HashMap<String, String>]) -> Vec<Document> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            let mut merged_doc = template.clone();
            for section in &mut merged_doc.sections {
                for elem in &mut section.elements {
                    if let BlockElement::Paragraph(p) = elem {
                        for inline in &mut p.inlines {
                            match inline {
                                InlineElement::Field(field) => {
                                    if let FieldType::MergeField(field_name) = &field.field_type {
                                        if let Some(val) = record.get(field_name) {
                                            field.cached_value = val.clone();
                                        }
                                    }
                                }
                                InlineElement::Run(run) => {
                                    // Also check for chevrons inside raw text: «FieldName»
                                    for (k, v) in record {
                                        let placeholder = format!("«{}»", k);
                                        if run.text.contains(&placeholder) {
                                            run.text = run.text.replace(&placeholder, v);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            results.push(merged_doc);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_merge() {
        let mut doc = Document::new("Letter", "Author");
        let mut p = Paragraph::new("p1");
        p.add_run(TextRun::new("Dear «CustomerName», your invoice is «Amount»."));
        doc.add_paragraph(p);

        let mut row1 = HashMap::new();
        row1.insert("CustomerName".to_string(), "Acme Corp".to_string());
        row1.insert("Amount".to_string(), "$1,250.00".to_string());

        let mut row2 = HashMap::new();
        row2.insert("CustomerName".to_string(), "Wayne Enterprises".to_string());
        row2.insert("Amount".to_string(), "$4,500.00".to_string());

        let merged = MailMergeEngine::merge_records(&doc, &[row1, row2]);
        assert_eq!(merged.len(), 2);

        let BlockElement::Paragraph(p1) = &merged[0].sections[0].elements[0] else { panic!() };
        assert_eq!(p1.plain_text(), "Dear Acme Corp, your invoice is $1,250.00.");

        let BlockElement::Paragraph(p2) = &merged[1].sections[0].elements[0] else { panic!() };
        assert_eq!(p2.plain_text(), "Dear Wayne Enterprises, your invoice is $4,500.00.");
    }
}
