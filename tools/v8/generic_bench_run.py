"""Execute interleaved benchmark aggregates and require numerical plus world safety."""
import hashlib
import time
from pathlib import Path
from typing import Literal

from generic_bench import (
    AuditError,
    Product,
    git_bytes,
    justified_ratio,
    require,
    sha256,
    validate_products,
    write_new,
)
from generic_bench_safety import Aggregate, audit_bench_safety
from generic_process import ProcessSpec, execute
from mux2_gate_inputs import Json
from read_gate_checks import wait_for_load


def run_bench(base: Product, candidate: Product, purpose: str, script: Path,
              script_sha256: str, output: Path,
              justifications: tuple[dict[str, str], ...] = ()) -> dict[str, Json]:
    """Execute sequential admitted arms. Caller owns the global measurement lock.

    Build receipts must be generated/authenticated by the generic source runner,
    not accepted from an untrusted CLI. Source cleanliness and effective data
    inventories are checked by that runner before and after this invocation.
    """
    validate_products(base, candidate, purpose)
    require(bool(base.input_files) and bool(candidate.input_files), 'authenticated runtime input inventories required')
    require(script.is_absolute() and script.resolve() == script, 'canonical harness input required')
    require(sha256(script) == script_sha256, 'command input changed')
    require(output.is_absolute() and output.parent.resolve() == output.parent, 'canonical fresh output required')
    require(not any(output.is_relative_to(product.root) for product in (base, candidate)),
            'evidence output must be outside products')
    output.mkdir(exist_ok=False)
    changelog_bytes = git_bytes(candidate.root, 'show', f'{candidate.source_sha}:CHANGELOG.md')
    changelog = changelog_bytes.decode()
    # Validate every supplied explanation even if all observed ratios later pass.
    _ = justified_ratio(1, 1, 'n200', 'mean', justifications, changelog)
    invocations: list[Json] = []
    aggregates: list[Aggregate] = []
    try:
        arms: tuple[tuple[Literal['base', 'candidate'], Product], ...] = (('base', base), ('candidate', candidate))
        for repeat in range(1, 4):
            for arm, product in arms:
                validate_products(base, candidate, purpose)
                require(sha256(script) == script_sha256, 'command input drift')
                triplet = wait_for_load()
                run_dir = output / f'{arm}-r{repeat}'
                argv = [str(product.binary), 'bench', '--out', str(run_dir), '--commands', str(script),
                        '--repeats', '1', '--commit', product.source_sha]
                runtime_inputs = dict(product.input_files)
                external_inputs: tuple[tuple[Path, str], ...] = ()
                if script.is_relative_to(product.root):
                    relative = script.relative_to(product.root).as_posix()
                    require(relative in runtime_inputs and runtime_inputs[relative] == script_sha256,
                            'script missing from authenticated product inventory')
                else:
                    external_inputs = ((script, script_sha256),)
                spec = ProcessSpec(tuple(argv), product.root, product.binary, product.binary_sha256,
                                   product.root, tuple(sorted(runtime_inputs.items())), run_dir,
                                   (('WORLDSIM_GIT_SHA', product.source_sha), ('WORLDSIM_GATE_OBSERVER', '1')),
                                   external_inputs=external_inputs)
                record: dict[str, Json] = {'arm': arm, 'repeat': repeat, 'argv': [part for part in argv],
                    'cwd': str(product.root), 'source_sha': product.source_sha,
                    'binary_sha256': product.binary_sha256, 'script_sha256': script_sha256,
                    'build_receipt_sha256': product.build_receipt_sha256,
                    'load_1_5_15': list(triplet), 'started_unix': time.time(),
                    'exit_code': None, 'finished_unix': None}
                write_new(output / f'{arm}-r{repeat}.started.json', record)
                result = execute(spec)
                record.update(exit_code=result.native_exit, finished_unix=time.time(),
                              producer_pid=result.pid, process_status=result.status,
                              process_receipt=str(result.receipt), process_receipt_sha256=sha256(result.receipt))
                record['stdout_sha256'] = sha256(run_dir / 'stdout.log')
                record['stderr_sha256'] = sha256(run_dir / 'stderr.log')
                write_new(output / f'{arm}-r{repeat}.finished.json', record)
                invocations.append(record)
                require(result.status == 'PASS' and result.native_exit == 0,
                        f'{arm} round {repeat}: producer status {result.status}, exit {result.native_exit}')
                aggregates.append(Aggregate(arm, repeat, product, spec, result, sha256(result.receipt)))
                validate_products(base, candidate, purpose)
                require(sha256(script) == script_sha256, 'command input drift')
        audit = audit_bench_safety(output, tuple(aggregates), purpose, script, script_sha256, justifications)
        audit['invocations'] = invocations
        audit['purpose'] = purpose
        audit['changelog_sha256'] = hashlib.sha256(changelog_bytes).hexdigest()
        write_new(output / 'result.json', audit)
        return audit
    except (AuditError, ValueError, KeyError, OSError, UnicodeError) as exc:
        write_new(output / 'failure.json', {'status': 'FAIL_OR_INCOMPLETE', 'reason': str(exc),
                                          'invocations': invocations})
        raise
