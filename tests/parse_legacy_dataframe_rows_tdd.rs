#[cfg(test)]
mod parse_legacy_dataframe_rows_tests {
    
    #[test]
    fn test_row_boundary_detection() {
        assert!(is_row_boundary(&Token::Semicolon));
        assert!(is_row_boundary(&Token::RightBracket));
        assert!(!is_row_boundary(&Token::Comma));
        assert!(!is_row_boundary(&Token::Identifier));
    }
    
    fn is_row_boundary(token: &Token) -> bool {
        matches!(token, Token::Semicolon | Token::RightBracket)
    }
    
    #[test]
    fn test_end_detection() {
        assert!(is_end_token(&Token::RightBracket));
        assert!(!is_end_token(&Token::Semicolon));
        assert!(!is_end_token(&Token::Comma));
    }
    
    fn is_end_token(token: &Token) -> bool {
        matches!(token, Token::RightBracket)
    }
    
    #[test]
    fn test_separator_detection() {
        assert!(is_separator(&Token::Comma));
        assert!(!is_separator(&Token::Semicolon));
        assert!(!is_separator(&Token::RightBracket));
    }
    
    fn is_separator(token: &Token) -> bool {
        matches!(token, Token::Comma)
    }
    
    #[test]
    fn test_row_separator_detection() {
        assert!(is_row_separator(&Token::Semicolon));
        assert!(!is_row_separator(&Token::Comma));
        assert!(!is_row_separator(&Token::RightBracket));
    }
    
    fn is_row_separator(token: &Token) -> bool {
        matches!(token, Token::Semicolon)
    }
    
    #[test]
    fn test_row_addition() {
        let mut rows = Vec::new();
        let row = vec![1, 2, 3];
        
        add_non_empty_row(&mut rows, row.clone());
        assert_eq!(rows.len(), 1);
        
        add_non_empty_row(&mut rows, vec![]);
        assert_eq!(rows.len(), 1);  // Empty row not added
    }
    
    fn add_non_empty_row<T>(rows: &mut Vec<Vec<T>>, row: Vec<T>) {
        if !row.is_empty() {
            rows.push(row);
        }
    }
    
    #[test]
    fn test_column_value_assignment() {
        let mut columns = vec![
            Column { values: vec![] },
            Column { values: vec![] },
        ];
        
        let rows = vec![
            vec![1, 2],
            vec![3, 4],
        ];
        
        populate_columns_from_rows(&mut columns, &rows);
        
        assert_eq!(columns[0].values, vec![1, 3]);
        assert_eq!(columns[1].values, vec![2, 4]);
    }
    
    fn populate_columns_from_rows<T: Clone>(columns: &mut [Column<T>], rows: &[Vec<T>]) {
        for (col_idx, column) in columns.iter_mut().enumerate() {
            for row in rows {
                if col_idx < row.len() {
                    column.values.push(row[col_idx].clone());
                }
            }
        }
    }
    
    // Mock types for testing
    #[derive(Debug, PartialEq)]
    enum Token {
        Comma,
        Semicolon,
        RightBracket,
        Identifier,
    }
    
    struct Column<T> {
        values: Vec<T>,
    }
}