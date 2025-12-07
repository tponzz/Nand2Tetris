#include <iostream>

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

    std::string input = argv[1];
    std::string output = argv[2];

    std::cout << "vm" << std::endl;
    std::cout << "in: " << input << std::endl;
    std::cout << "out: " << output << std::endl;
    return 0;
}
