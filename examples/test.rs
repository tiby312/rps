


#[derive(Debug)]
enum Res{
    Equal,
    Attack,
    Avoid
}



//returns whether or not to attack or avoid b from the perspective of a.
fn func(a:u8,b:u8)->Res{
    if a==b
    {
        Res::Equal
    }
    else
    {
        if (a & (b<<3)) != 0
        {
            Res::Attack
        }
        else
        {
            Res::Avoid
        }
    }
}


fn main(){
    let rock    =0b100001;
    let paper   =0b001010;
    let scissor =0b010100;

    //avoid
    println!("{:?}",func(rock,paper));
    println!("{:?}",func(scissor,rock));
    println!("{:?}",func(paper,scissor));

    println!();

    //attack
    println!("{:?}",func(paper,rock));
    println!("{:?}",func(rock,scissor));
    println!("{:?}",func(scissor,paper));

    println!();

    //equal
    println!("{:?}",func(rock,rock));
    println!("{:?}",func(scissor,scissor));
    println!("{:?}",func(paper,paper));

}