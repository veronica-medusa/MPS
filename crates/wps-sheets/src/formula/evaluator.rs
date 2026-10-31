use super::functions::call_function;
use super::parser::{BinaryOp, FormulaExpr, UnaryOp};
use crate::cell::{CellCoordinate, CellError, CellValue};
use crate::sparse_matrix::SparseMatrix;

pub struct Evaluator<'a> {
    matrix: &'a SparseMatrix,
}

impl<'a> Evaluator<'a> {
    pub fn new(matrix: &'a SparseMatrix) -> Self {
        Self { matrix }
    }

    pub fn eval(&self, expr: &FormulaExpr) -> CellValue {
        match expr {
            FormulaExpr::Literal(val) => val.clone(),
            FormulaExpr::Cell(coord) => {
                self.matrix.get_cell(*coord).map(|c| c.value.clone()).unwrap_or(CellValue::Empty)
            }
            FormulaExpr::Range(range) => {
                let mut rows = Vec::new();
                for r in range.start.row..=range.end.row {
                    let mut row_vals = Vec::new();
                    for c in range.start.col..=range.end.col {
                        let coord = CellCoordinate::new(r, c);
                        let val = self.matrix.get_cell(coord).map(|cell| cell.value.clone()).unwrap_or(CellValue::Empty);
                        row_vals.push(val);
                    }
                    rows.push(row_vals);
                }
                CellValue::Array(rows)
            }
            FormulaExpr::Spill(coord) => {
                self.matrix.get_cell(*coord).map(|c| c.value.clone()).unwrap_or(CellValue::Empty)
            }
            FormulaExpr::Unary(op, inner) => {
                let val = self.eval(inner);
                match op {
                    UnaryOp::Neg => {
                        val.as_number().map(|n| CellValue::Number(-n)).unwrap_or(CellValue::Error(CellError::Value))
                    }
                    UnaryOp::Pos => {
                        val.as_number().map(CellValue::Number).unwrap_or(CellValue::Error(CellError::Value))
                    }
                }
            }
            FormulaExpr::Binary(op, lhs, rhs) => {
                let left_val = self.eval(lhs);
                let right_val = self.eval(rhs);
                self.eval_binary(*op, &left_val, &right_val)
            }
            FormulaExpr::FunctionCall { name, args } => {
                let eval_args: Vec<CellValue> = args.iter().map(|a| self.eval(a)).collect();
                call_function(name, &eval_args)
            }
        }
    }

    fn eval_binary(&self, op: BinaryOp, left: &CellValue, right: &CellValue) -> CellValue {
        match op {
            BinaryOp::Add => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Number(l + r),
                    _ => CellValue::Error(CellError::Value),
                }
            }
            BinaryOp::Sub => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Number(l - r),
                    _ => CellValue::Error(CellError::Value),
                }
            }
            BinaryOp::Mul => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Number(l * r),
                    _ => CellValue::Error(CellError::Value),
                }
            }
            BinaryOp::Div => {
                match (left.as_number(), right.as_number()) {
                    (Some(_), Some(r)) if r == 0.0 => CellValue::Error(CellError::DivZero),
                    (Some(l), Some(r)) => CellValue::Number(l / r),
                    _ => CellValue::Error(CellError::Value),
                }
            }
            BinaryOp::Power => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Number(l.powf(r)),
                    _ => CellValue::Error(CellError::Value),
                }
            }
            BinaryOp::Concat => {
                CellValue::Text(format!("{}{}", left.as_string(), right.as_string()))
            }
            BinaryOp::Eq => CellValue::Boolean(left == right),
            BinaryOp::NotEq => CellValue::Boolean(left != right),
            BinaryOp::Lt => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Boolean(l < r),
                    _ => CellValue::Boolean(left.as_string() < right.as_string()),
                }
            }
            BinaryOp::LtEq => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Boolean(l <= r),
                    _ => CellValue::Boolean(left.as_string() <= right.as_string()),
                }
            }
            BinaryOp::Gt => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Boolean(l > r),
                    _ => CellValue::Boolean(left.as_string() > right.as_string()),
                }
            }
            BinaryOp::GtEq => {
                match (left.as_number(), right.as_number()) {
                    (Some(l), Some(r)) => CellValue::Boolean(l >= r),
                    _ => CellValue::Boolean(left.as_string() >= right.as_string()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::parser::Parser;
    use crate::formula::token::Lexer;

    #[test]
    fn test_evaluator() {
        let mut matrix = SparseMatrix::new();
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Number(100.0)); // A1
        matrix.set_value(CellCoordinate::new(1, 0), CellValue::Number(50.0));  // A2

        let mut lexer = Lexer::new("=A1 + A2 * 2");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        let evaluator = Evaluator::new(&matrix);
        let result = evaluator.eval(&ast);

        assert_eq!(result, CellValue::Number(200.0));
    }

    #[test]
    fn test_py_formula_and_dependencies() {
        let matrix = SparseMatrix::new();
        let mut lexer = Lexer::new("=PY(\"wp.range('A1:B2').describe()\", \"object\")");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        // 1. Dependency extraction should find A1, A2, B1, B2
        let deps = ast.extract_dependencies();
        assert_eq!(deps.len(), 4);
        assert!(deps.contains(&CellCoordinate::new(0, 0))); // A1
        assert!(deps.contains(&CellCoordinate::new(0, 1))); // B1
        assert!(deps.contains(&CellCoordinate::new(1, 0))); // A2
        assert!(deps.contains(&CellCoordinate::new(1, 1))); // B2

        // 2. Evaluator should call PY
        let evaluator = Evaluator::new(&matrix);
        let result = evaluator.eval(&ast);
        assert_eq!(result, CellValue::Text("[🐍 PythonObject]".to_string()));
    }
}
