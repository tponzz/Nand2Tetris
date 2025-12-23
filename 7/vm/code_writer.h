#ifndef VM_CODE_WRITER_HH

#include <fstream>
#include <string>

#include "parser.h"

namespace Vm {

class CodeWriter
{
  public:
    CodeWriter(const std::string& filepath);
    ~CodeWriter();

    void WriteArithmetic(const std::string& cmd_line);
    void WritePushPop( Parser::Cmd cmd,
                      const std::string& seg,
                      const size_t idx);
    void Close();

  private:
    std::ofstream _out;
};

} // namespace Vm

#define VM_CODE_WRITER_HH
#endif