(fun_function_definition
  name: (identifier) @name)
(fun_function_definition
  name: (identifier (identifier) @name))

(imp_function_definition
  name: (identifier) @name)
(imp_function_definition
  name: (identifier (identifier) @name))
(parameters
  (identifier) @name)

(object_definition
  name: (identifier) @name)
(object_definition
  name: (identifier (identifier) @name))
(object_field
  (identifier) @name)
(object_field
  (identifier (identifier) @name))

(imp_type_definition
  name: (identifier) @name)
(imp_type_definition
  name: (identifier (identifier) @name))
(imp_type_constructor
  (identifier) @name)
(imp_type_constructor
  (identifier (identifier) @name))
(imp_type_constructor_field
  (identifier) @name)
(imp_type_constructor_field
  (identifier (identifier) @name))

(fun_type_definition
  name: (identifier) @name)
(fun_type_definition
  name: (identifier (identifier) @name))
(fun_type_constructor
  (identifier) @name)
(fun_type_constructor
  (identifier (identifier) @name))
(fun_type_constructor_fields
  (identifier) @name)
(fun_type_constructor_fields
  (identifier (identifier) @name))

(hvm_definition
  name: (identifier) @name)
(hvm_definition
  name: (identifier (identifier) @name))

(constructor
  (identifier) @name)
(constructor
  (identifier (identifier) @name))
