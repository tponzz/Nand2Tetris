#include <filesystem>
#include <iostream>
#include <ranges>
#include <string>
#include <vector>

#include "code_writer.h"
#include "parser.h"

static void
Usage()
{
    std::cout << "Usage: vm <input.vm>\n";
    std::cout << "  input.vm  : vm code 1\n";
}

// path/to/file.ext -> path/to/file
static std::string
PathToFilename(const std::string& path)
{
    namespace fs = std::filesystem;
    fs::path rel = fs::relative(path);

    if (fs::is_directory(rel)) {
        return rel / *(--rel.end());
    } else {
        return rel.parent_path() / rel.stem();
    }
}

static std::vector<std::string>
GetVmFiles(const std::string& path)
{
    namespace fs = std::filesystem;
    fs::path rel = fs::relative(path);

    std::vector<std::string> paths{};
    if (fs::is_directory(rel)) {
        auto is_vmfile = [](auto&& f) { return f.path().extension() == ".vm"; };

        for (auto vm_file : fs::directory_iterator{ rel } | std::views::filter(is_vmfile)) {
            paths.push_back(vm_file.path());
        }
    } else {
        if (rel.extension() == ".vm") {
            paths.push_back(rel);
        }
    }

    return paths;
}

static std::string
GetStem(const std::string& path)
{
    namespace fs = std::filesystem;
    return fs::path(path).stem();
}

int
main(int argc, char const* argv[])
{
    if (argc != 2) {
        Usage();
        return -1;
    }

    std::string in_path = argv[1];

    // dir or .vm
    std::vector<std::string> target_vm = GetVmFiles(in_path);
    if (target_vm.size() == 0) {
        Usage();
        return -1;
    }

    std::string out_path = PathToFilename(in_path).append(".asm");

    // a single CodeWriter for the whole run
    Vm::CodeWriter writer{ out_path };

    for (auto&& path : target_vm) {
        Vm::Parser p{ path };

        // for static variables
        writer.SetFileName(GetStem(path));

        std::cout << "in: " << path << std::endl;
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
                case Vm::Parser::Cmd::Invalid: {
                    std::cerr << "Invalid command: " << static_cast<int>(type) << "\n";
                } break;

                default:
                    break;
            }
        }
    }

    std::cout << "out: " << out_path << std::endl;
    std::cout << "Transration Finished" << std::endl;

    return 0;
}
