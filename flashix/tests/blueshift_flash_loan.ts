import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlueshiftFlashLoan } from "../target/types/blueshift_flash_loan";

describe("blueshift_flash_loan", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.blueshiftFlashLoan as Program<BlueshiftFlashLoan>;

  it("Say Hello!", async () => {
    const tx = await program.methods.sayHello().rpc();
    console.log("Your transaction signature", tx);
  });
});
