from http.server import BaseHTTPRequestHandler, HTTPServer
import logging
import json
import os
from inspect import getmembers, isfunction

UNITTEST = "UNIT"
SIMPLETEST = "SIMPLE"

MODULE_TESTS = "tests"

SUCCESS = "SUCCESS"
FAILED = "FAILED"
TIMEOUT = "TIMEOUT"
EVALUATION_ERROR = "EVALUATION_ERROR"


class Worker(BaseHTTPRequestHandler):
    def _set_response(self):
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.end_headers()

    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        post_data = self.rfile.read(content_length)
        # logging.info("POST request,\nPath: %s\nHeaders:\n%s\n\nBody:\n%s\n",
        #              str(self.path), str(self.headers), post_data.decode('utf-8'))

        if self.path == "/evaluate":
            test_result = self.test_submission(json.loads(post_data.decode()))
            #logging.info(f"RESULT {test_result}")
        self._set_response()
        self.wfile.write(json.dumps(test_result, indent=2).encode('utf-8'))

    def test_submission(self, eval_data):
        if eval_data["testtype"] == UNITTEST:
            return self.check_with_unit_test(eval_data)
        if eval_data["testtype"] == SIMPLETEST:
            return self.check_with_simple_test(eval_data)
        return f"{EVALUATION_ERROR}: testtype unknown: {eval_data['testtype']}"

    def check_with_simple_test(self, eval_data):
        solution = eval_data["sourceCode"]
        tests = self.importCode(eval_data["tests"], MODULE_TESTS)
        try: 
            (feedback, result) = tests.run(solution)
            errors = [(str(test), output)
                        for (test, output) in result.errors]
            failures = [(str(test), output)
                        for (test, output) in result.failures]

            return {"result-type": self.checkResultType(result),
                    "runs": result.testsRun,
                    "errors": errors,
                    "failures": failures,
                    "feedback": feedback}
        except:
            return f"{EVALUATION_ERROR}: Test script has no run function."

    def check_with_unit_test(self, eval_data):
        global solution
        global tests
        solution = self.importCode(eval_data["sourceCode"], "solution")
        tests = self.importCode(eval_data["tests"], "tests")

        # Alle Funktionen aus solution fuer den import nach tests.run() vorbereiten.
        functions = {}
        for (key, func) in [o for o in getmembers(solution) if isfunction(o[1])]:
            functions[key] = func
        try:
            (feedback, result) = tests.run(functions)
            errors = [(str(test), output) for (test, output) in result.errors]
            failures = [(str(test), output)
                        for (test, output) in result.failures]

            return {"result-type": self.checkResultType(result),
                    "runs": result.testsRun,
                    "errors": errors,
                    "failures": failures,
                    "feedback": feedback}
        except:
            return f"{EVALUATION_ERROR}: Test script has no run function."

    def checkResultType(self, result) -> str:
        if len(result.errors) == len(result.failures) == 0:
            return SUCCESS
        if True in [_failure_timeout(output) for (_, output) in result.failures]:
            return TIMEOUT
        else:
            return FAILED

    def importCode(self, code, name, add_to_sys_modules=0):
        import types
        module = types.ModuleType(name)

        if add_to_sys_modules:
            import sys
            sys.modules[name] = module
        exec(code, module.__dict__)
        return module


def _failure_timeout(failure: str) -> bool:
    return "timeout_decorator.timeout_decorator.TimeoutError: 'Timed Out'" in failure


def run(server_class=HTTPServer, handler_class=Worker, port=8080):
    logging.basicConfig(level=logging.INFO)
    server_address = ('', port)
    httpd = server_class(server_address, handler_class)
    logging.info('Starting worker...\n')
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    httpd.server_close()
    logging.info('Stopping worker...\n')


if __name__ == '__main__':
    from sys import argv

    if len(argv) == 2:
        run(port=int(argv[1]))
    else:
        run()
