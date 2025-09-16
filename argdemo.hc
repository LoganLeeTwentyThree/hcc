module Main =
	(* import each arg as a function with this signature *)
	import arg0 : std::integer -> std::integer = args::arg0

	(* @title: byte_to_string
		@signature: integer -> string
		@description: giant match statement (｡﹏｡")@ *)
	let byte_to_string = fn byte => match byte with
    	|48 => "0"
    	|49 => "1"
    	|50 => "2"
    	|51 => "3"
    	|52 => "4"
    	|53 => "5"
    	|54 => "6"
    	|55 => "7"
    	|56 => "8"
    	|57 => "9"
    	|65 => "A"
    	|66 => "B"
    	|67 => "C"
    	|68 => "D"
    	|69 => "E"
    	|70 => "F"
    	|71 => "G"
    	|72 => "H"
    	|73 => "I"
    	|74 => "J"
    	|75 => "K"
    	|76 => "L"
    	|77 => "M"
    	|78 => "N"
    	|79 => "O"
   		|80 => "P"
    	|81 => "Q"
    	|82 => "R"
    	|83 => "S"
    	|84 => "T"
    	|85 => "U"
    	|86 => "V"
   		|87 => "W"
    	|88 => "X"
    	|89 => "Y"
    	|90 => "Z"
    	|97  => "a"
    	|98  => "b"
    	|99  => "c"
    	|100 => "d"
    	|101 => "e"
    	|102 => "f"
    	|103 => "g"
    	|104 => "h"
    	|105 => "i"
    	|106 => "j"
    	|107 => "k"
    	|108 => "l"
    	|109 => "m"
    	|110 => "n"
    	|111 => "o"
    	|112 => "p"
    	|113 => "q"
    	|114 => "r"
    	|115 => "s"
    	|116 => "t"
    	|117 => "u"
    	|118 => "v"
    	|119 => "w"
    	|120 => "x"
    	|121 => "y"
    	|122 => "z"
    	|_ => "?"

	(* @title: byte_list_to_string
		@signature: (list::t integer) -> string
		@description: turns a list of bytes into a string@ *)
	let byte_list_to_string = fn list =>
		let inner = fn index => match (list::nth index list) with
		| opt::None => ""
		| opt::Some of a => string::concatenate (byte_to_string a) (inner index+1)
		in inner 0

	(* @title: decode_to_bytes
		@signature: (integer -> integer) -> (list::t integer)
		@description: gets each byte of arg in order and returns a list of them, reversed due to some errors@ *)
	let decode_to_bytes = 
	fn op => 
		let inner = fn index =>
			match op index with 
			| 0 => list::Nil
			| b => (inner index+1) |> list::push b
		in inner 0

	let decode_to_string = fn op => decode_to_bytes op |> byte_list_to_string
	
	(* decode arg0 into a string and print it *)
	let () = decode_to_string arg0 |> string::print
end
