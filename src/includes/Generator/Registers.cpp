#include <Generator/Registers.hpp>
#include "Registers.hpp"

Register::Register(REGISTER reg) : reg(reg) { }

Register::REGISTER Register::get() {
  return this->reg;
}

Register Register::to64() {
  return convertTo(0);
}

Register Register::to32() {
  return convertTo(1);
}

Register Register::to16() {
  return convertTo(2);
}

Register Register::to08() {
  return convertTo(3);
}

Register Register::promote() {
  switch (getInt() / BLOCK_WIDTH) {
    case 0 : Errors::warn("Register already 64bit"); return *this;
    case 1 : return to64();
    case 2 : return to32();
    case 3 : return to16();
    default: throw std::out_of_range("Invalid Register Index");
  }
}

Register Register::demote() {
  switch (getInt() / BLOCK_WIDTH) {
    case 0 : return to32();
    case 1 : return to16();
    case 2 : return to08();
    case 3 : Errors::warn("Register already 8bit"); return *this;
    default: throw std::out_of_range("Invalid Register Index");
  }
}

std::string Register::toString() {
  switch (this->get()) {
    case REGISTER::RAX : return "rax";
    case REGISTER::RCX : return "rcx"; 
    case REGISTER::RDX : return "rdx";
    case REGISTER::RSI : return "rsi";
    case REGISTER::RDI : return "rdi";
    case REGISTER::R8  : return "r8";
    case REGISTER::R9  : return "r9";
    case REGISTER::R10 : return "r10";
    case REGISTER::R11 : return "r11";
    case REGISTER::R12 : return "r12";
    case REGISTER::R13 : return "r13";
    case REGISTER::R14 : return "r14";
    case REGISTER::R15 : return "r15";

    case REGISTER::EAX : return "eax";
    case REGISTER::ECX : return "ecx";
    case REGISTER::EDX : return "edx";
    case REGISTER::ESI : return "esi";
    case REGISTER::EDI : return "edi";
    case REGISTER::R8D : return "r8d";
    case REGISTER::R9D : return "r9d";
    case REGISTER::R10D : return "r10d";
    case REGISTER::R11D : return "r11d";
    case REGISTER::R12D : return "r12d";
    case REGISTER::R13D : return "r13d";
    case REGISTER::R14D : return "r14d";
    case REGISTER::R15D : return "r15d";

    case REGISTER::AX : return "ax";
    case REGISTER::CX : return "cx";
    case REGISTER::DX : return "dx";
    case REGISTER::SI : return "si";
    case REGISTER::DI : return "di";
    case REGISTER::R8W : return "r8w";
    case REGISTER::R9W : return "r9w";
    case REGISTER::R10W : return "r10w";
    case REGISTER::R11W : return "r11w";
    case REGISTER::R12W : return "r12w";
    case REGISTER::R13W : return "r13w";
    case REGISTER::R14W : return "r14w";
    case REGISTER::R15W : return "r15w";

    case REGISTER::AL : return "al";
    case REGISTER::CL : return "cl";
    case REGISTER::DL : return "dl";
    case REGISTER::SIL : return "sil";
    case REGISTER::DIL : return "dil";
    case REGISTER::R8B : return "r8b";
    case REGISTER::R9B : return "r9b";
    case REGISTER::R10B : return "r10b";
    case REGISTER::R11B : return "r11b";
    case REGISTER::R12B : return "r12b";
    case REGISTER::R13B : return "r13b";
    case REGISTER::R14B : return "r14b";
    case REGISTER::R15B : return "r15b";

    default: return "";
  }
}

int Register::getInt() {
  return static_cast<int>(this->reg);
}

int Register::asInt(REGISTER reg) {
  return static_cast<int>(reg);
}

Register::REGISTER Register::asReg(int i) {
  if (i < asInt(REGISTER::RAX) || i > asInt(REGISTER::R15B))
    throw std::out_of_range("Invalid register index");
  return static_cast<REGISTER>(i);
}

Register Register::convertTo(int index) {
  int i = getInt();
  if (i / BLOCK_WIDTH == index) return *this;
  return asReg((i % BLOCK_WIDTH) + (index * BLOCK_WIDTH));
}
