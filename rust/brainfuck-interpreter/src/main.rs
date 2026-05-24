use std::io::Bytes;

fn build_bucket_map(program :&str)->Vec<usize>{
    let bytes = program.as_bytes();
    let length = program.len();
    let mut stack:Vec<usize> = Vec::new();
    let mut map = vec![0usize;length];

    for i in 0..length{

        match bytes[i] {
            b'[' => {
                stack.push(i);
            }

            b']' =>{
                let open = stack.pop().expect("Unmatched ]");
                map[open] = i;
                map[i] = open
            }

            _ =>{

            }
        }
    }

        if !stack.is_empty(){
            panic!("Unmatched ]")
        }
return map;
}

fn main() {
    let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

    let mut memory = [0u8;30000];
    let mut dp: usize = 0;
    let mut pc: usize = 0;

    let bytes = program.as_bytes();

    let map = build_bucket_map(program);

    while pc < bytes.len(){
        match bytes[pc] {
            b'>' =>{
                dp = dp +1;
            }
            b'<' =>{
                dp = dp -1;
            }
            b'+' =>{
                memory[dp] = memory[dp]+1;
            }
            b'-' =>{
                memory[dp] = memory[dp]-1;
            }
            b'.' =>{
                let  asi:char = memory[dp] as char;
                print!("{}",asi)
            }
            b',' =>{
                let input = bytes[pc];
                memory[dp] = input;
            }
            b'[' => {
                if memory[dp] == 0{
                    pc = map[pc]
                }
            }

            b']' =>{
                if memory[dp] != 0{
                    pc = map[pc]
                }
            }

            _ =>{

            }
        }
        pc = pc +1;
    }

}



