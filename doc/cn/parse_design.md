## Parser-Design

### 解析表达式
- shaun-rs 使用普拉特解析法来解析表达式, 普拉特解析法将运算符分为前缀运算符和中缀运算符, 这不难理解:
  - 前缀运算符, 是指该运算符接受单个的表达式进行运算, 例如 `!`, `+`, `-` 这些运算符都可以单独放在一个表达式之前来进行计算.
    中缀运算符则是指接受两个的运算符进行远算, 常见的 `+`, `-`, `*`, `/`, `>`, `>=` 等等这些都是中缀运算符, 中缀运算符接受两个表达式来进行运算.
  - 普拉特解析法对于每个注册了的前缀运算符和后缀运算符都需要注册对应的解析函数, 在进行解析的时候, 去判断当前是否有对应的前缀远算符解析函数, 如果有则进行解析, 没有则返回( 这里返回并不会在一开始就返回, 一般而言都是在递归调用的时候返回).
  - ```SQL
    SELECT -(1 + 20) AND -(1 + 30);
    ```
  - 这里的 `-(1 + 20)` 中的 `-` 是一个前缀运算符, 前缀运算符将后面的表达式看作一个整体, - 运算符是一个最上层的运算符, 即给 * (-1). 而 `(1 + 20)`这是一个整体, 普拉特解析法将数字同样也标记为前缀运算符, 数字会在 parser 这里转化为一个内部表达式, 因为 Token 那里保存的是一个字符串, `Token(Number(type: String))` => `Expression::Literal(Literal::())`.
  - ```SQL
    SELECT -(1 + 20) AND -(1 + 30);
    ```
    在经过 Parser 之后会成为一个 SelectStmt, 如:
    ```Rust
      SelectStmt(
        selects:
        Expression::Operation(Operation::And(
            Expression::Operatin(Operation::Substract(
                Expression::Operation(Operation::Add(
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(20)),
                ))
            )),
            Expression::Operation(Operation::Substract(
                Expression::Operation(Operation::Add(
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(30)),
                ))
            ))
        ))
        // 略
      )
      /**
       * 表达式树:
       *            And
       *         -      -
       *       +         +
       *     1   20    1   30
       */
    ```
  - 解析流程:
    lexer 产生的 TokenVec(实际上是流式的):
    ```rust
     Token::Keyword(Keyword::Select),
     Token::Minus,
     Token::LeftParen,
     Token::Number(1),
     Token::Add,
     Token::Number(20),
     Token::RightParen,
     Token::Keyword(Keyword::And),
     Token::Minus,
     Token::LeftParen,
     Token::Number(1),
     Token::Add,
     Token::Number(30),
     Token::RightParen,
     Token::Semicolon,
    ```
    - 第一个产生的 Token 是一个 Keyword(Select) 进入 parse_select_stmt,
    - 解析表达式(伪代码)
      ```
      Token::LeftParen
        if is_prefix_oper
          => parse_prefix_expr
            return parse_expression
              Token::Number(1) => Expression::Literal(Literal::Int(1))
              Token::Add
                if is_infix_oper
                  => parse_infix_expr
                     while pre_token != Token::Semiclon
                        && peek_token_predence > parse_expression_predence
                       return Expresssion::Add(
                          Expression::Literal(Literal::Int(1)),
                          Expression::Literal(Literal::Int(20)),
                       )
      Token::And(CurrentToken)
        if is_infix_oper
          => parse_infix_expr
            return Expression::And(
              Expression::Add(
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(20)),
              )
              parser.parse_expression(),
            )
            Token::LeftParen
              if is_prefix_oper
                => parse_prefix_expr
                  return parse_expression
                  Token::Number(1) => Expression::Literal(Literal::Int(1)),
                  Token::Add
                    if is_infix_oper
                      => parse_infix_expr
                        while pre_token != Token::Semiclon
                          && peek_token_predence > parse_expression_predence
                          return Expression::Add(
                            Expression::Literal(Literal::Int(1)),
                            Expression::Literal(Literal::Int(30)),
                          )
      ```
      在上面的伪代码中:
      ```rust
      while pre_token != Token::Semiclon
         && peek_token_predence > parse_expression_predence
      ```
      这里前面的 `pre_token != Token::Semiclon` 不难理解, 如果还没结束则继续解析, 但是
      `peek_token_predence > parse_expresion_predence` 这里是为什么呢?
      
      请考虑:
      ```SQL
      SELECT 1 + 20 + 30 + 40;
      ```
      这里的 `1 + 20 + 30 + 40` 只有一个运算符, 他们都是同级运算符, 在某一次递归时, 进入到中缀运算符 `+` 的parse函数中, 此时由于 peek_token 是 Token::Add, 那么他的优先级和我进入 parse_expression 所携带的优先级一样高, 我此时应该返回, 1 + 20 看成一个整体, 在向上递归的时候则会变成这样:
      ```rust
      //    +(parse_infix_expression)
      //  +    
      // 1 20
      ```
      ```SQL
      SELECT 1 + 20 * 30;
      ```
      此处 `*` 运算符的优先级高于 `+`, 当进入到
      ```Rust
        Expression::Add(
          Expression::Literal(Literal::Int(1)),
          parse_expression(Predence::Prefix)? // 这里以前缀优先符的级别进入 parse.
        )
      =>
        pre_token: Token::Number(20)
        peek_token: Token::Asterisk ('*' 运算符)
        满足 peek_token_predence > parse_expression_predence(Predence::Prefix)
      ```
      此时 parse_expression 函数会将 20 * 30 看作一个整体, 继续 parse, 再递归返回.


### 引用:

[1] 一种易于理解的解析规则 ([普拉特解析法 Vaughan R.Pratt Massachusetts Institute of Technology 1973 ](https://dl.acm.org/doi/pdf/10.1145/512927.512931))
