"""Black-box regression tests for playthrough's RPC functions, without a server.

The stub models websocat's documented text/binary prefix and base64 options.
Runtime smoke with the actual pinned websocat is separate evidence.
Run: python3 -m unittest discover -s scripts/tests -p 'test_playthrough_rpc.py' -v
"""
import base64
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / 'playthrough.sh'
FUNCTIONS = '\n'.join(re.findall(r'^[a-z0-9_]+\(\) \{\n.*?^\}', SCRIPT.read_text(), re.M | re.S))
STUB = '''#!/usr/bin/env python3
import base64, json, os, sys, time
args = sys.argv[1:]
def option(name):
    return args[args.index(name) + 1] if name in args else ''
request_line = sys.stdin.readline().rstrip('\\n')
if '--base64-text' in args:
    assert request_line.startswith(option('--text-prefix'))
    request_line = base64.b64decode(request_line[len(option('--text-prefix')):], validate=True).decode()
request = json.loads(request_line)
assert request['jsonrpc'] == '2.0'
fixture = json.load(open(os.environ['RPC_FIXTURE']))
for frame in fixture['frames']:
    binary = frame['kind'] == 'binary'
    payload = base64.b64decode(frame['data']) if binary else frame['data'].encode()
    if ('--base64' if binary else '--base64-text') in args:
        payload = base64.b64encode(payload)
    prefix = option('--binary-prefix' if binary else '--text-prefix').encode()
    sys.stdout.buffer.write(prefix + payload + b'\\n')
    sys.stdout.buffer.flush()
if fixture['stall']:
    time.sleep(5)
sys.exit(fixture['exit'])
'''


def text(value):
    return {'kind': 'text', 'data': json.dumps(value)}


def reply(request_id=2, **fields):
    return text({'jsonrpc': '2.0', 'id': request_id, **(fields or {'result': {'population': 5, 'tick': 154}})})


class PlaythroughRpcTests(unittest.TestCase):
    def call(self, frames, *, stall=False, exit_code=0, mode='rpc', timeout='0.5', early_exit=False):
        with tempfile.TemporaryDirectory(prefix='playthrough-rpc-') as directory:
            directory = Path(directory)
            fixture = directory / 'frames.json'
            fixture.write_text(json.dumps({'frames': frames, 'stall': stall, 'exit': exit_code}))
            stub = directory / 'websocat'
            stub.write_text('#!/usr/bin/env bash\nexit 7\n' if early_exit else STUB)
            stub.chmod(0o700)
            env = os.environ | {'PATH': str(directory) + ':' + os.environ['PATH'],
                                'RPC_FIXTURE': str(fixture), 'RPC_TEST_MODE': mode,
                                'RPC_TIMEOUT_SECONDS': timeout, 'RPC_TEST_PARENT_DELAY': str(int(early_exit))}
            shell = '''set -u
set -o pipefail
CIV_WS_HOST=127.0.0.1
CIV_SERVER_PORT=3000
CIV_WS_PATH=/ws
log() { :; }
ok() { :; }
fail() { printf '%s\\n' "$*" >&2; }
''' + FUNCTIONS + '''
# Model the parent being descheduled just after starting a coprocess.
if [[ "$RPC_TEST_PARENT_DELAY" == 1 ]]; then
    set -T
    trap 'if [[ "$BASH_COMMAND" == "local rpc_pid="* ]]; then sleep 0.1; fi' DEBUG
fi
if [[ "$RPC_TEST_MODE" == health ]]; then
    ws_health
    status=$?
    jq -n --argjson status "$status" '{status:$status}'
elif [[ "$RPC_TEST_MODE" == step2 ]]; then
    step2_attach_client
    status=$?
    jq -n --argjson status "$status" '{status:$status}'
else
    body="$(rpc 2 sim.status '{}')"
    status=$?
    valid="$(json_ok "$body")"
    jq -n --arg body "$body" --arg valid "$valid" --argjson status "$status" '{body:$body,valid:$valid,status:$status}'
fi
'''
            result = subprocess.run(['bash', '-c', shell], env=env, capture_output=True, text=True, timeout=4)
            self.assertEqual(result.returncode, 0, result.stderr)
            return json.loads(result.stdout), result.stderr

    def test_matching_result_only(self):
        result, _ = self.call([reply()])
        self.assertEqual(result['status'], 0)
        self.assertEqual(json.loads(result['body'])['result']['population'], 5)
        self.assertEqual(result['valid'], 'true')

    def test_mixed_notifications_binary_and_matching_result(self):
        delta = b'{"VoxelDelta":{"tick":155,"deltas":[]}}'
        binary = b'F3D0\x00' + len(delta).to_bytes(4, 'big') + delta
        result, errors = self.call([
            text({'jsonrpc': '2.0', 'method': 'scene.reset', 'params': {'tick': 155}}),
            {'kind': 'binary', 'data': base64.b64encode(binary).decode()},
            text({'VoxelDelta': {'tick': 155, 'deltas': []}}), reply(),
            {'kind': 'binary', 'data': base64.b64encode(b'F3D0\x00\n\xff').decode()},
        ])
        self.assertEqual(result['status'], 0, errors)
        self.assertEqual(json.loads(result['body']), json.loads(reply()['data']))
        self.assertEqual(result['valid'], 'true')
        self.assertNotIn('null byte', errors)
        self.assertNotIn('parse error', errors)

    def test_unrelated_error_and_string_id_are_ignored(self):
        result, _ = self.call([reply(99, error={'code': -32601, 'message': 'other'}), reply('2'), reply()])
        self.assertEqual(result['status'], 0)
        self.assertEqual(json.loads(result['body'])['id'], 2)
        self.assertEqual(result['valid'], 'true')

    def test_pretty_printed_response_is_one_message(self):
        response = {'jsonrpc': '2.0', 'id': 2, 'result': {'population': 5, 'label': 'F3D0\nquoted'}}
        result, _ = self.call([{'kind': 'text', 'data': json.dumps(response, indent=2)}])
        self.assertEqual(result['status'], 0)
        self.assertEqual(json.loads(result['body']), response)

    def test_matching_error_is_preserved_and_fails(self):
        result, _ = self.call([reply(error={'code': -32602, 'message': 'bad params', 'data': {'field': 'seed'}})])
        self.assertNotEqual(result['status'], 0)
        self.assertEqual(json.loads(result['body'])['error']['code'], -32602)
        self.assertEqual(result['valid'], 'false')

    def test_matching_malformed_envelopes_fail(self):
        for response in (
            {'jsonrpc': '1.0', 'id': 2, 'result': {}},
            {'jsonrpc': '2.0', 'id': 2},
            {'jsonrpc': '2.0', 'id': 2, 'result': {}, 'error': {'code': -1, 'message': 'bad'}},
            {'jsonrpc': '2.0', 'id': 2, 'error': None},
            {'jsonrpc': '2.0', 'id': 2, 'error': {'code': 'bad', 'message': 'bad'}},
            {'jsonrpc': '2.0', 'id': 2, 'error': {'code': -1}},
        ):
            with self.subTest(response=response):
                result, errors = self.call([text(response)])
                self.assertNotEqual(result['status'], 0)
                self.assertEqual(result['valid'], 'false')
                self.assertTrue(errors)

    def test_invalid_text_frame_fails_explicitly(self):
        result, errors = self.call([{'kind': 'text', 'data': '{not JSON'}])
        self.assertNotEqual(result['status'], 0)
        self.assertTrue(errors)

    def test_eof_without_matching_response_fails(self):
        for frames in ([], [reply(99)], [text({'jsonrpc': '2.0', 'method': 'scene.reset'})]):
            with self.subTest(frames=frames):
                result, errors = self.call(frames)
                self.assertNotEqual(result['status'], 0)
                self.assertEqual(result['body'], '')
                self.assertTrue(errors)

    def test_transport_failure_is_not_success(self):
        result, errors = self.call([], exit_code=7)
        self.assertNotEqual(result['status'], 0)
        self.assertTrue(errors)

    def test_fast_transport_exit_with_delayed_parent_is_reported(self):
        result, errors = self.call([], early_exit=True)
        self.assertNotEqual(result['status'], 0)
        self.assertEqual(result['body'], '')
        self.assertIn('exit 7', errors)
        self.assertNotIn('unbound variable', errors)

    def test_timeout_without_matching_reply_fails(self):
        result, errors = self.call([reply(99)], stall=True, timeout='0.1')
        self.assertNotEqual(result['status'], 0)
        self.assertEqual(result['body'], '')
        self.assertTrue(errors)

    def test_returns_when_matching_reply_arrives_before_timeout(self):
        started = time.monotonic()
        result, errors = self.call([reply()], stall=True, timeout='3')
        self.assertLess(time.monotonic() - started, 1.5)
        self.assertEqual(result['status'], 0, errors)
        self.assertEqual(json.loads(result['body'])['id'], 2)

    def test_health_requires_successful_matching_status(self):
        for frames in ([], [reply(99)], [reply(0, error={'code': -32601, 'message': 'no method'})]):
            with self.subTest(frames=frames):
                result, _ = self.call(frames, mode='health')
                self.assertNotEqual(result['status'], 0)
        result, _ = self.call([reply(0)], mode='health')
        self.assertEqual(result['status'], 0)

    def test_step2_rejects_rpc_error(self):
        result, _ = self.call([reply(error={'code': -32602, 'message': 'bad'})], mode='step2')
        self.assertEqual(result['status'], 2)


if __name__ == '__main__':
    unittest.main()
