### Variable Initialization
Variable types will be inferred.
    
    var integer = 5
    var number = 5.5
    var boolean = true|false
    var character = 'a'
    var string = "abc"
    var tuple = ("abc", 5)

Access the elements of an anonymous tuple with their index.
    
    var other_string = tuple.0
    var other_int = tuple.1

### Variable declaration (without initialization)
If a variable is not initialized, the type must be annotated explicitely.
    
    var integer: int
    var number: num 
    var boolean: bool
    var character: char
    var string: str
    var tuple: (str, int)

### Named Tuples (Structs)
Structs are basically enhanced tuples with member names.
    
    var person = (age: 52, name: "Alex")
    var num = person.age

You can define a struct with type anotations, just like any variable.
    
    var person: (age: num, name: str)

It is useful to declare your structs as types for reusability and understandability.
    
    type Person = (age: num, name: str)
    var person: Person = (52, "Alex")     
    // If the explicit type anotation is not made, "person" will be treated as an anonymous tuple
    print(person.age)

### Enums
Enums are a way to tell the compiler that a variable can only have a number of values.

    type StrBool = "true" | "false"

    // Correct
    var a: StrBool = "true"
    
    // Compiler error
    var a: StrBool = "1"

Enums can also be named.  
Names will be treated as aliases for the real value.
    
    type StrBool = T -> "true" | F -> "false"
    var a: StrBool = StrBool.T
    // a = "true"

You can use these names to check for conditions.

    if a = StrBool.T => print("a is true")
    else             => print("a is false")

Or [match](#match) each value

### Lists
    var int_list = [5, 6, 7, 8]
    var num_list = [5, 5.5, 8.8]
    
    var str_list: [str]
    var tuple_list: [(str, bool)]
    tuple_list = [("true", true), ("false", false)]

To add things to a list, use the + operator.
    
    var int_list: [int]
    int_list += 5
    int_list = int_list + 5

You can also add multiple lists, but they must be of the same type.
    
    var list = [1, 2, 3]
    var other = [4, 5, 6]
    list += other                   
    // list = [1, 2, 3, 4, 5, 6]

To remove an element from a list, use the - operator.
    
    var list = [1, 2, 3]
    list -= 1                       
    // list = [2, 3]

You can substract multiple lists, basically removing the first intersection.
    
    var list = [1, 2, 3]
    var other = [2, 3]
    list -= other                   
    // list = [1]

The remove operator will only remove the first occurrence, if you want to remove all of them, use the -- operator.
    
    var list = [1, 2, 3, 1, 2, 3]
    list --= 1                      
    // list = [2, 3, 2, 3]
    var other = [1, 2]
    list --= other                  
    // list = [3, 3]

To access a specific element on a list, use [index].
    
    var list = ["hello", "world"]
    print(list[1])                  
    // output -> "world"

If an out-of-bounds error is detected (trying to access an element that isn't there) the program will be terminated at runtime.
    
    print(list[2])                  
    // output -> ERROR: Out-of-bounds in line X, "list[2]".

You can define the starting size of a list.
    
    var list: [int, 3]

### Dictionaries
Dictionaries act like lists, but instead of using the index of insertion to access each element, you use a custom defined one.
    
    var dictionary = [ "true" -> true, "false" -> false ]
    print(dictionary["true"])
    // output -> true

To declare an empty dictionary you must annotate its types explicitely
    
    var dictionary: [str -> bool]

> Depending on the flavor of Polyglot you're using, the implementation of the Dictionaries will change drastically, so don't assume constant lookup.  
> If the language in question has a native dictionary, then it will be used.

### Control flow
#### If / Else If / Else
If some condition is met, execute the code inside the `if`.  
If the condition is false, execute the code inside the `elif` (else if).  
If all the conditions above are false, execute the code inside the `else`.
    
    if condition
        // something
    end
    
    if condition
        // something
    elif condition
        // something else
    else
         // other things
    end

The "end" keyword can be omitted when only one expression is inside the if/elseif/else.
Also, in this case one-liners are supported, using "=>" next to the condition.
    
    // Omit "end"    
    if condition
        // something
    
    // One-liners
    if condition    => // something
    elif condition  => // something
    else            => // something
    
    // NOT VALID
    if condition // something

You can chain conditions with the "&&" and "||" operators
    
    // Check if a > 0 AND a < 5
    if a > 0 && a < 5
        print("a is in the (0, 5) range")
    end
    
    // Check if a < 0 OR a > 0
    if a < 0 || a > 0
        print("a is not 0")
    end

#### Match
The match pattern allows to check if a variable has certain compile-time values.  
You can indicate an "if any of these" using "_". 
    
    var number = 5 
    match number
        1 => print("one")
        2 => print("two")
        3 => print("three")
        4 => print("four")
        5 => print("five")
        _ => print("number not defined")

On most languages it is basically identical to an if / else if / else
    
    var number = 5
    if   number = 1 => print("one")
    elif number = 2 => print("two")
    elif number = 3 => print("three")
    elif number = 4 => print("four")
    elif number = 5 => print("five")
    else            => print("number not defined")

> Do not assume that match will be translated as a lookup table, it is flavor dependent.  
> If the language has a similar implementation (`switch` in C/C++, `match` in Rust) then match will be transpiled into this implementation.

You can write multiple lines on each block, but then the pattern must finish with an "end".
    
    match number
        1 => number += 1
             print("one")
        2 => number += 2
             print("two")
        ...
    end

You can match against multiple values.
    
    match number
        1 | 2 | 3 => print("one, two or three")
        _         => print("other numbers") 

### Loops
    while condition
        // something
    end
    
    for i in 0..100
        // something
    end
    
    for item in list
        // something
    end

### Functions
If a function does take any argument, parenthesis can be skipped.  
Indentation is not necessary, but it is strongly recommended.
    
    fn void
        // something
    end
    
    fn void_args(a: int, b: num, ...)
        // something
    end

Then to call it (note that the parenthesis can not be skipped here)

    void()
    void_args(5, 6)
    
To return something from a function you need to anotate it with the return type.

    fn return_no_args: int
        67
    end
    
    fn return_args(a: int, b: num, ...): string
        // something
        "hello"
    end


You can return multiple things using a tuple, return order must be preserved.
    
    fn return_multiple: (int, num, char)
        (25, 3.14, 'a')
    end

To return early, use the "ret" keyword. The last line in a function will always be treated as an implicit return.
    
    fn return_early(a: num, b: num): int
        if a > b -> ret a - b
        else if b > a -> ret b - a
        else -> 0
    end


To take a tuple as an argument, use parenthesis in between the type annotation.
    
    fn tuple_args(tuple: (int, num, char))
        // something
    end


You can also pass named tuples.
    
    fn tuple_named_args(struct: (a: int, b: num, c: char))
        // something
    end

Tuples can be destructured when passed as args, so a more idiomatic way to write the above expression would be:
    
    fn tuple_idiomatic_args(a: int, b: num, c: char)
        // something
    end
    
    tuple = (89, 99.99, 'z')
    tuple_idiomatic_args(tuple)

### Generics
#### Type defined generics
You can use generics to define functions for multiple types at the same time.
    
    type Addable = int | num | str | char
    fn add(a: Addable, b: Addable): Addable
        a + b
    end
    
    add(5, 6)
    // Return: 11
    
    add("5", "6")
    // Return: "56"

Note that all args of `add` must be of the same type, as `Addable`'s type is defined with the first argument.  

#### Enum defined generics
If you want to define an implementation for all the possible combinations of types, use an enum as the type of the arguments.

    fn add(a: int|str, b: int|str): str
        a + b
    end

    add(5, 6)
    // Return: "11" (note that it is a string now)

    add("Numbers: ", 88))
    // Return: "Numbers: 88"

The return type must be defined (it cannot be generic), as you need to ensure what you're doing is legal (you cannot add two `str` and return an `int`).

Generics are just syntactic sugar for monomorphisation, the compiler will create an implementation for each type used.  
The above function would be transpiled into:

    fn add_int_int(a: int, b: int): str
        str(a + b)
    end
    
    fn add_str_str(a: str, b: str): str
        a + b
    end
    
    fn add_int_str(a: int, b: str): str
        a + b
    end

    fn add_str_int(a: str, b: int): str
        a + b
    end

So avoid using generic enums for a lot of types, as the number of implementations will increase a lot.  
[Type defined generics](#type-defined-generics) are much preferred, as they only generate one implementation per type used.

### Expressions
If/elif/else/match are expressions for Polyglot, meaning that you can assign them to variables.
If you're familiar with the Rust programming language, the behaviour is the same.

	var str_bool = "true"
	var parsed_bool: bool = 	
		match
		    "true"  => true
		    "false" => false
		end

	// parsed_bool = true

All branches must return values of the same type, as a variable cannot be of multiple types.

### Basic operations
    var number = 10
    var ops: num
    
    // Addition
    ops = 5 + 6
    ops = number + 15
    ops += 65
    
    // Substraction
    ops = 5 - 6
    ops = 15 - number
    ops -= 65
    
    // Multiplication
    ops = 5 * 6
    ops = number * 15
    ops *= 65
    
    // Division
    ops = 5 / 6
    ops = number / 15
    ops /= 65

    // Modulus
    ops = 5 % 6
    ops = number % 15
    ops %= 65
    
    // Exponentiation
    ops = 5^6
    ops = number ^ 15
    ops ^= 65

### Comparisons

    // Equal
    if a = b

    // Not equal
    if a != b
    
    // Smaller
    if a < b
    
    // Smaller or equal
    if a <= b

    // Larger
    if a > b

    // Larger or equal
    if a >= b

    // Negation
    if !a

### Built-in functions
#### Type conversions
Type conversions are used to transform a type into another.  
All conversions are structured as `end_type(value)`
Not all conversions are valid, and some will throw a runtime error.
    
    var number = 682
    var string = str(number)
    // string = "682"

The inverse process is also valid
    
    var string = "682"
    var number = int(string)
    // number = 682

But converting a string that isn't a number will fail

    var string = "hey!"
    var number = int(string)
    // ERROR: "hey!" can not be parsed into an int

Conversions can also be applied to lists/dictionaries

    var bool_list = [true, false, false, true]
    var int_list = int(bool_list)
    // int_list = [1, 0, 0, 1]

    var dict: [int -> bool] = [5 -> true, -8 -> false]
    var str_dict: [str -> str] = str(dict)
    // str_dict = ["5" -> "true", "-8" -> false]
    

    // You can also specify if you want to convert the keys or the values
    keys_dict = str(dict, keys)
    // str_dict = ["5" -> true, "-8" -> false]
    
    values_dict = str(dict, values)
    // values_dict = [5 -> "true", -8 -> "false"]

#### Print
Prints the contents of a variable to the terminal.
    
    var a = 5
    print(a)
    // Output: 5

You can operate inside the print argument (useful for concatenating strings).

    var a = 5
    print("a is equal to" + a)
    // Output: "a is equal to 5"

#### Dbg
You can use it to quickly debug some value