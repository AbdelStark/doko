# Witness program hash mistmatch bug context

## Bitcoin Core implementation 

SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH = "Witness program hash mismatch"

```c++
static bool VerifyWitnessProgram(const CScriptWitness& witness, int witversion, const std::vector<unsigned char>& program, unsigned int flags, const BaseSignatureChecker& checker, ScriptError* serror, bool is_p2sh)
{
    CScript exec_script; //!< Actually executed script (last stack item in P2WSH; implied P2PKH script in P2WPKH; leaf script in P2TR)
    std::span stack{witness.stack};
    ScriptExecutionData execdata;

    if (witversion == 0) {
        if (program.size() == WITNESS_V0_SCRIPTHASH_SIZE) {
            // BIP141 P2WSH: 32-byte witness v0 program (which encodes SHA256(script))
            if (stack.size() == 0) {
                return set_error(serror, SCRIPT_ERR_WITNESS_PROGRAM_WITNESS_EMPTY);
            }
            const valtype& script_bytes = SpanPopBack(stack);
            exec_script = CScript(script_bytes.begin(), script_bytes.end());
            uint256 hash_exec_script;
            CSHA256().Write(exec_script.data(), exec_script.size()).Finalize(hash_exec_script.begin());
            if (memcmp(hash_exec_script.begin(), program.data(), 32)) {
                return set_error(serror, SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH);
            }
            return ExecuteWitnessScript(stack, exec_script, flags, SigVersion::WITNESS_V0, checker, execdata, serror);
        } else if (program.size() == WITNESS_V0_KEYHASH_SIZE) {
            // BIP141 P2WPKH: 20-byte witness v0 program (which encodes Hash160(pubkey))
            if (stack.size() != 2) {
                return set_error(serror, SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH); // 2 items in witness
            }
            exec_script << OP_DUP << OP_HASH160 << program << OP_EQUALVERIFY << OP_CHECKSIG;
            return ExecuteWitnessScript(stack, exec_script, flags, SigVersion::WITNESS_V0, checker, execdata, serror);
        } else {
            return set_error(serror, SCRIPT_ERR_WITNESS_PROGRAM_WRONG_LENGTH);
        }
    } else if (witversion == 1 && program.size() == WITNESS_V1_TAPROOT_SIZE && !is_p2sh) {
        // BIP341 Taproot: 32-byte non-P2SH witness v1 program (which encodes a P2C-tweaked pubkey)
        if (!(flags & SCRIPT_VERIFY_TAPROOT)) return set_success(serror);
        if (stack.size() == 0) return set_error(serror, SCRIPT_ERR_WITNESS_PROGRAM_WITNESS_EMPTY);
        if (stack.size() >= 2 && !stack.back().empty() && stack.back()[0] == ANNEX_TAG) {
            // Drop annex (this is non-standard; see IsWitnessStandard)
            const valtype& annex = SpanPopBack(stack);
            execdata.m_annex_hash = (HashWriter{} << annex).GetSHA256();
            execdata.m_annex_present = true;
        } else {
            execdata.m_annex_present = false;
        }
        execdata.m_annex_init = true;
        if (stack.size() == 1) {
            // Key path spending (stack size is 1 after removing optional annex)
            if (!checker.CheckSchnorrSignature(stack.front(), program, SigVersion::TAPROOT, execdata, serror)) {
                return false; // serror is set
            }
            return set_success(serror);
        } else {
            // Script path spending (stack size is >1 after removing optional annex)
            const valtype& control = SpanPopBack(stack);
            const valtype& script = SpanPopBack(stack);
            if (control.size() < TAPROOT_CONTROL_BASE_SIZE || control.size() > TAPROOT_CONTROL_MAX_SIZE || ((control.size() - TAPROOT_CONTROL_BASE_SIZE) % TAPROOT_CONTROL_NODE_SIZE) != 0) {
                return set_error(serror, SCRIPT_ERR_TAPROOT_WRONG_CONTROL_SIZE);
            }
            execdata.m_tapleaf_hash = ComputeTapleafHash(control[0] & TAPROOT_LEAF_MASK, script);
            if (!VerifyTaprootCommitment(control, program, execdata.m_tapleaf_hash)) {
                return set_error(serror, SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH);
            }
            execdata.m_tapleaf_hash_init = true;
            if ((control[0] & TAPROOT_LEAF_MASK) == TAPROOT_LEAF_TAPSCRIPT) {
                // Tapscript (leaf version 0xc0)
                exec_script = CScript(script.begin(), script.end());
                execdata.m_validation_weight_left = ::GetSerializeSize(witness.stack) + VALIDATION_WEIGHT_OFFSET;
                execdata.m_validation_weight_left_init = true;
                return ExecuteWitnessScript(stack, exec_script, flags, SigVersion::TAPSCRIPT, checker, execdata, serror);
            }
            if (flags & SCRIPT_VERIFY_DISCOURAGE_UPGRADABLE_TAPROOT_VERSION) {
                return set_error(serror, SCRIPT_ERR_DISCOURAGE_UPGRADABLE_TAPROOT_VERSION);
            }
            return set_success(serror);
        }
    } else if (!is_p2sh && CScript::IsPayToAnchor(witversion, program)) {
        return true;
    } else {
        if (flags & SCRIPT_VERIFY_DISCOURAGE_UPGRADABLE_WITNESS_PROGRAM) {
            return set_error(serror, SCRIPT_ERR_DISCOURAGE_UPGRADABLE_WITNESS_PROGRAM);
        }
        // Other version/size/p2sh combinations return true for future softfork compatibility
        return true;
    }
    // There is intentionally no return statement here, to be able to use "control reaches end of non-void function" warnings to detect gaps in the logic above.
}

bool VerifyScript(const CScript& scriptSig, const CScript& scriptPubKey, const CScriptWitness* witness, unsigned int flags, const BaseSignatureChecker& checker, ScriptError* serror)
{
    static const CScriptWitness emptyWitness;
    if (witness == nullptr) {
        witness = &emptyWitness;
    }
    bool hadWitness = false;

    set_error(serror, SCRIPT_ERR_UNKNOWN_ERROR);

    if ((flags & SCRIPT_VERIFY_SIGPUSHONLY) != 0 && !scriptSig.IsPushOnly()) {
        return set_error(serror, SCRIPT_ERR_SIG_PUSHONLY);
    }

    // scriptSig and scriptPubKey must be evaluated sequentially on the same stack
    // rather than being simply concatenated (see CVE-2010-5141)
    std::vector<std::vector<unsigned char> > stack, stackCopy;
    if (!EvalScript(stack, scriptSig, flags, checker, SigVersion::BASE, serror))
        // serror is set
        return false;
    if (flags & SCRIPT_VERIFY_P2SH)
        stackCopy = stack;
    if (!EvalScript(stack, scriptPubKey, flags, checker, SigVersion::BASE, serror))
        // serror is set
        return false;
    if (stack.empty())
        return set_error(serror, SCRIPT_ERR_EVAL_FALSE);
    if (CastToBool(stack.back()) == false)
        return set_error(serror, SCRIPT_ERR_EVAL_FALSE);

    // Bare witness programs
    int witnessversion;
    std::vector<unsigned char> witnessprogram;
    if (flags & SCRIPT_VERIFY_WITNESS) {
        if (scriptPubKey.IsWitnessProgram(witnessversion, witnessprogram)) {
            hadWitness = true;
            if (scriptSig.size() != 0) {
                // The scriptSig must be _exactly_ CScript(), otherwise we reintroduce malleability.
                return set_error(serror, SCRIPT_ERR_WITNESS_MALLEATED);
            }
            if (!VerifyWitnessProgram(*witness, witnessversion, witnessprogram, flags, checker, serror, /*is_p2sh=*/false)) {
                return false;
            }
            // Bypass the cleanstack check at the end. The actual stack is obviously not clean
            // for witness programs.
            stack.resize(1);
        }
    }

    // Additional validation for spend-to-script-hash transactions:
    if ((flags & SCRIPT_VERIFY_P2SH) && scriptPubKey.IsPayToScriptHash())
    {
        // scriptSig must be literals-only or validation fails
        if (!scriptSig.IsPushOnly())
            return set_error(serror, SCRIPT_ERR_SIG_PUSHONLY);

        // Restore stack.
        swap(stack, stackCopy);

        // stack cannot be empty here, because if it was the
        // P2SH  HASH <> EQUAL  scriptPubKey would be evaluated with
        // an empty stack and the EvalScript above would return false.
        assert(!stack.empty());

        const valtype& pubKeySerialized = stack.back();
        CScript pubKey2(pubKeySerialized.begin(), pubKeySerialized.end());
        popstack(stack);

        if (!EvalScript(stack, pubKey2, flags, checker, SigVersion::BASE, serror))
            // serror is set
            return false;
        if (stack.empty())
            return set_error(serror, SCRIPT_ERR_EVAL_FALSE);
        if (!CastToBool(stack.back()))
            return set_error(serror, SCRIPT_ERR_EVAL_FALSE);

        // P2SH witness program
        if (flags & SCRIPT_VERIFY_WITNESS) {
            if (pubKey2.IsWitnessProgram(witnessversion, witnessprogram)) {
                hadWitness = true;
                if (scriptSig != CScript() << std::vector<unsigned char>(pubKey2.begin(), pubKey2.end())) {
                    // The scriptSig must be _exactly_ a single push of the redeemScript. Otherwise we
                    // reintroduce malleability.
                    return set_error(serror, SCRIPT_ERR_WITNESS_MALLEATED_P2SH);
                }
                if (!VerifyWitnessProgram(*witness, witnessversion, witnessprogram, flags, checker, serror, /*is_p2sh=*/true)) {
                    return false;
                }
                // Bypass the cleanstack check at the end. The actual stack is obviously not clean
                // for witness programs.
                stack.resize(1);
            }
        }
    }
```