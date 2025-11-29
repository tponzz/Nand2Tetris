#include <fstream>
#include <string>

namespace Asm {

class Parser
{
  public:
    enum class Instruction
    {
        A = 0,
        C,
        L,
        Invalid
    };

    struct CInstrunction
    {
        std::string d;
        std::string c;
        std::string j;
    };

    explicit Parser(const std::string& filepath);

    ~Parser();

    bool HasMoreLines();
    void Advance();
    Instruction InstructionType() const;
    std::string Symbol() const;
    std::string Dest() const;
    std::string Comp() const;
    std::string Jump() const;

    std::string Current() const;

  private:
    std::ifstream in_;
    std::string cur_;
    std::string d_, c_, j_;
};

} // namespace Asm