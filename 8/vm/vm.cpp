#include <filesystem>
#include <iostream>

#include "code_writer.h"
#include "parser.h"

void
Usage()
{
    std::cout << "Usage: vm <input.vm>\n";
    std::cout << "  input.vm  : vm code 1\n";
}

// path/to/file.ext -> path/to/file
std::string
PathToFilename(const std::string& path)
{
    namespace fs = std::filesystem;
    fs::path abs = fs::relative(path);

    return abs.parent_path() / abs.stem();
}

int
main(int argc, char const* argv[])
{
    if (argc != 2) {
        Usage();
        return -1;
    }

    std::string vm_txt  = argv[1];
    std::string asm_txt = PathToFilename(vm_txt).append(".asm");

    std::cout << "vm" << std::endl;
    std::cout << "in: " << vm_txt << std::endl;
    std::cout << "out: " << asm_txt << std::endl;

    Vm::Parser p{ vm_txt };
    Vm::CodeWriter writer{ asm_txt };

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
            case Vm::Parser::Cmd::Label: {
                writer.WriteLabel(p.Arg1());
            } break;
            case Vm::Parser::Cmd::Goto: {
                writer.WriteGoto(p.Arg1());
            } break;
            case Vm::Parser::Cmd::If: {
                writer.WriteIf(p.Arg1());
            } break;
            case Vm::Parser::Cmd::Function: {
                writer.WriteFuntion(p.Arg1(), p.Arg2());
            } break;
            case Vm::Parser::Cmd::Call: {
                writer.WriteCall(p.Arg1(), p.Arg2());
            } break;
            case Vm::Parser::Cmd::Return: {
                writer.WriteReturn();
            } break;

            default:
                break;
        }
    }

    std::cout << "Transration Finished" << std::endl;

    return 0;
}
