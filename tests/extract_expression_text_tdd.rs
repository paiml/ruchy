#[cfg(test)]
mod extract_expression_text_tests {
    
    #[derive(Default)]
    struct ExprContext {
        in_string: bool,
        in_char: bool,
        escaped: bool,
        brace_count: i32,
    }
    
    #[test]
    fn test_string_delimiter_handling() {
        let mut ctx = ExprContext::default();
        assert!(!ctx.in_string);
        
        handle_string_delimiter(&mut ctx);
        assert!(ctx.in_string);
        
        handle_string_delimiter(&mut ctx);
        assert!(!ctx.in_string);
    }
    
    fn handle_string_delimiter(ctx: &mut ExprContext) {
        ctx.in_string = !ctx.in_string;
    }
    
    #[test]
    fn test_char_delimiter_handling() {
        let mut ctx = ExprContext::default();
        assert!(!ctx.in_char);
        
        handle_char_delimiter(&mut ctx);
        assert!(ctx.in_char);
        
        handle_char_delimiter(&mut ctx);
        assert!(!ctx.in_char);
    }
    
    fn handle_char_delimiter(ctx: &mut ExprContext) {
        ctx.in_char = !ctx.in_char;
    }
    
    #[test]
    fn test_brace_counting() {
        let mut ctx = ExprContext::default();
        assert_eq!(ctx.brace_count, 0);
        
        handle_open_brace(&mut ctx);
        assert_eq!(ctx.brace_count, 1);
        
        handle_open_brace(&mut ctx);
        assert_eq!(ctx.brace_count, 2);
        
        handle_close_brace(&mut ctx);
        assert_eq!(ctx.brace_count, 1);
    }
    
    fn handle_open_brace(ctx: &mut ExprContext) {
        ctx.brace_count += 1;
    }
    
    fn handle_close_brace(ctx: &mut ExprContext) {
        ctx.brace_count -= 1;
    }
    
    #[test]
    fn test_escape_handling() {
        let mut ctx = ExprContext::default();
        assert!(!ctx.escaped);
        
        handle_backslash(&mut ctx);
        assert!(ctx.escaped);
        
        reset_escape(&mut ctx, 'x');
        assert!(!ctx.escaped);
    }
    
    fn handle_backslash(ctx: &mut ExprContext) {
        ctx.escaped = true;
    }
    
    fn reset_escape(ctx: &mut ExprContext, ch: char) {
        if ch != '\\' {
            ctx.escaped = false;
        }
    }
    
    #[test]
    fn test_should_process_char() {
        let ctx = ExprContext {
            in_string: false,
            in_char: false,
            escaped: false,
            brace_count: 0,
        };
        
        assert!(should_process_string_quote(&ctx));
        assert!(should_process_char_quote(&ctx));
        assert!(should_process_brace(&ctx));
    }
    
    fn should_process_string_quote(ctx: &ExprContext) -> bool {
        !ctx.in_char && !ctx.escaped
    }
    
    fn should_process_char_quote(ctx: &ExprContext) -> bool {
        !ctx.in_string && !ctx.escaped
    }
    
    fn should_process_brace(ctx: &ExprContext) -> bool {
        !ctx.in_string && !ctx.in_char
    }
    
    #[test]
    fn test_termination_condition() {
        let ctx = ExprContext {
            brace_count: 0,
            ..Default::default()
        };
        
        assert!(should_terminate(&ctx));
        
        let ctx = ExprContext {
            brace_count: 1,
            ..Default::default()
        };
        
        assert!(!should_terminate(&ctx));
    }
    
    fn should_terminate(ctx: &ExprContext) -> bool {
        ctx.brace_count == 0
    }
}