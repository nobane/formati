mod test_formati {
    use formati::format;
    use std::f32::consts;

    #[test]
    fn test_formati_basic() {
        let answer = 42;
        let result = format!("The answer is {answer}");
        assert_eq!(result, "The answer is 42");
    }

    #[test]
    fn test_formati_dotted_access() {
        let user = (String::from("Alice"), 30);
        let result = format!("Name: {user.0}, Age: {user.1}");
        assert_eq!(result, "Name: Alice, Age: 30");
    }

    #[test]
    fn test_complex_expressions() {
        // Test ::
        let result = format!(
            "Created: {String::from(\"Alice\")}, \
         Length: {String::from(\"Alice\").len()}, \
         Uppercase: {String::from(\"Alice\").to_uppercase()}, \
         Vec: {Vec::<i32>::new().len()}",
        );

        assert_eq!(
            result,
            "Created: Alice, Length: 5, Uppercase: ALICE, Vec: 0",
        );

        // Test with indexing
        let numbers = [1, 2, 3];
        let result2 = format!("Numbers: {numbers.len():04}, First: {numbers[0]:02}");
        assert_eq!(result2, "Numbers: 0003, First: 01");
    }

    #[test]
    fn test_formati_formatting_specs() {
        let value = consts::PI;
        let result = format!("Pi: {value:.2}");
        assert_eq!(result, "Pi: 3.14");
    }

    #[test]
    fn test_formati_repeated_expression() {
        let person = (String::from("Bob"), 25);
        let result = format!("Name: {person.0}, Info: {person.0} is {person.1} years old");
        assert_eq!(result, "Name: Bob, Info: Bob is 25 years old");
    }

    #[test]
    fn test_formati_nested_structures() {
        let data = ((1, 2), (3, 4));
        let result = format!("Values: {data.0.0}, {data.0.1}, {data.1.0}, {data.1.1}");
        assert_eq!(result, "Values: 1, 2, 3, 4");
    }

    #[test]
    fn test_formati_with_named_args() {
        let x = 10;
        let y = 20;
        let result = format!("{x} + {y} = {sum}", sum = x + y);
        assert_eq!(result, "10 + 20 = 30");
    }

    #[test]
    fn test_formati_escaped_braces() {
        let result = format!("{{escaped}} but {not}", not = "interpolated");
        assert_eq!(result, "{escaped} but interpolated");
    }

    #[test]
    fn test_formati_struct_fields_and_methods() {
        struct Employee {
            id: u32,
            salary: i32,
            department: String,
        }

        impl Employee {
            fn get_title(&self) -> &str {
                "Software Engineer"
            }

            fn format_id(&self) -> String {
                format!("EMP-{:04}", self.id)
            }
        }

        let employee = Employee {
            id: 157,
            salary: 85000,
            department: String::from("Engineering"),
        };

        let project_data = (2023, "Project Formati");

        // Test accessing struct fields, method calls, and repeated expressions
        let result = format!(
            "Employee ID: {employee.id}, Department: {employee.department}\n\
         Salary: ${employee.salary}, Title: {employee.get_title()}\n\
         Formatted ID: {employee.format_id()}, Employee ID again: {employee.id}\n\
         Project Year: {project_data.0}, Project Name: {project_data.1}",
        );

        assert_eq!(
            result,
            "Employee ID: 157, Department: Engineering\n\
         Salary: $85000, Title: Software Engineer\n\
         Formatted ID: EMP-0157, Employee ID again: 157\n\
         Project Year: 2023, Project Name: Project Formati"
        );

        // Verify that method calls are handled correctly
        let method_result = format!("Employee: {employee.format_id()} - {employee.get_title()}");
        assert_eq!(method_result, "Employee: EMP-0157 - Software Engineer");

        // Test with formatting specifiers
        let detail_result = format!(
        "ID: {employee.id:04}, Salary: {employee.salary:+}, Department: {employee.department:.5}"
    );
        assert_eq!(detail_result, "ID: 0157, Salary: +85000, Department: Engin");
    }
}
