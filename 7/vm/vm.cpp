#include <iostream>

#include "code_writer.h"
#include "parser.h"

void
Usage()
{
    std::cout << "Usage: ./vm <input.vm> <output.vm>\n";
    std::cout << "  input.vm  : vm code 1\n";
    std::cout << "  output.vm : vm code 2\n";
}

int
main(int argc, char const* argv[])
{
    if (argc != 3) {
        Usage();
        return -1;
    }

    std::string input  = argv[1];
    std::string output = argv[2];

    std::cout << "vm" << std::endl;
    std::cout << "in: " << input << std::endl;
    std::cout << "out: " << output << std::endl;

    Vm::Parser p{ input };
    Vm::CodeWriter writer{ output };

    while (p.HasMoreLines()) {
        p.Advance();

        const auto type = p.CommandType();
        switch (type) {
            case Vm::Parser::Cmd::Pop:
            case Vm::Parser::Cmd::Push: {
                writer.WritePushPop(type, p.Arg1(), p.Arg2());
            } break;
            case Vm::Parser::Cmd::Arithmetic: {
                writer.WriteArithmetic(p.Arg1());
            } break;

            default:
                break;
        }
    }

    std::cout << "Transration Finished" << std::endl;

    return 0;
}
