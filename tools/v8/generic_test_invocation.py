"""Closed invocation identities for four qualified integration-test families only."""
from dataclasses import dataclass
from typing import Literal

from generic_bench import Product, git_bytes, sha256
from generic_gate import document, require


@dataclass(frozen=True, slots=True)
class Invocation:
    step: int | None
    profile: Literal['debug','release']
    package: str
    target: str
    kind: str = 'integration_test'
    filter: str | None = None
    ignored: str = 'exclude'
    threads: int = 8
    capture: bool = True

    @property
    def argv(self) -> tuple[str, ...]:
        return (f'--test-threads={self.threads}',) + (() if self.capture else ('--nocapture',))


def invocations(name: str) -> tuple[Invocation, ...]:
    targets = ('save_load','colony_world','colony_commands','colony_jobs','colony_items')
    match name:
        case 'development':
            return tuple(Invocation(None,'release','sim-test',target,threads=1,capture=False) for target in targets)
        case 'gate04':
            return tuple(Invocation(4,'debug','sim-test',target) for target in targets)
        case 'gate05':
            return tuple(Invocation(5,'release','sim-test',target) for target in targets)
        case 'gate17':
            return tuple(Invocation(17,'release','sim-test',target) for target in ('colony_commands','colony_jobs','colony_world'))
        case 'gate24':
            return (Invocation(24,'release','sim-test','save_load'),)
        case _:
            raise ValueError(f'unregistered native invocation: {name}')


def verify_artifact(product: Product, profile: str) -> None:
    """Keep benchmark verification release-only; authenticate native debug separately."""
    import re

    require(profile in {'release','debug'}, '/profile', 'unsupported native profile')
    require(re.fullmatch('[0-9a-f]{40}',product.source_sha) is not None, '/source', 'full product SHA required')
    for digest in (product.binary_sha256,product.build_receipt_sha256):
        require(re.fullmatch('[0-9a-f]{64}',digest) is not None, '/artifact', 'full SHA256 required')
    for path in (product.root,product.binary,product.build_receipt):
        require(path.is_absolute() and path.resolve() == path, str(path), 'canonical artifact path required')
    require(product.binary.is_relative_to(product.root), '/binary', 'outside product build root')
    require(git_bytes(product.root,'rev-parse','HEAD').decode().strip() == product.source_sha, '/source', 'HEAD drift')
    require(sha256(product.binary) == product.binary_sha256, '/binary', 'binary changed')
    require(sha256(product.build_receipt) == product.build_receipt_sha256, '/artifact', 'receipt changed')
    receipt = document(product.build_receipt.read_bytes().decode())
    for key,value in (('source_sha',product.source_sha),('binary_sha256',product.binary_sha256),
                      ('profile',profile),('exit_code',0)):
        receipt.field(key).equals(value)
