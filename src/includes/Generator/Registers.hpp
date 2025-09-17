#pragma once

#include <iostream>
#include <Utils/Errors.hpp>

class Register {
  public:
    enum class REGISTER {
      RAX=0, RCX, RDX, RSI, RDI, R8, R9, R10, R11, R12, R13, R14, R15, 
      EAX, ECX, EDX, ESI, EDI, R8D, R9D, R10D, R11D, R12D, R13D, R14D, R15D,
      AX, CX, DX, SI, DI, R8W, R9W, R10W, R11W, R12W, R13W, R14W, R15W,
      AL, CL, DL, SIL, DIL, R8B, R9B, R10B, R11B, R12B, R13B, R14B, R15B
    };

    Register(REGISTER reg);
    REGISTER get();
    Register to64();
    Register to32();
    Register to16();
    Register to08();
    Register promote();
    Register demote();
    std::string toString();

  private:
    static constexpr int BLOCK_WIDTH = (static_cast<int>(REGISTER::R15) - static_cast<int>(REGISTER::RAX)) + 1;
    REGISTER reg;
    static int asInt(REGISTER reg);
    static REGISTER asReg(int i);
    Register convertTo(int i);
    int getInt();
};
