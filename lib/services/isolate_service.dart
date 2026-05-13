import 'dart:async';
import 'dart:isolate';

typedef IsolateTask<T, R> = R Function(T param);

class IsolateService {
  static Future<R> run<T, R>(IsolateTask<T, R> task, T param) async {
    final receivePort = ReceivePort();
    await Isolate.spawn(
      _isolateEntry<T, R>,
      _IsolatePayload(task, param, receivePort.sendPort),
    );
    final result = await receivePort.first;
    if (result is _IsolateError) {
      throw Exception(result.message);
    }
    return result as R;
  }

  static Future<R> runWithProgress<T, R>(
    R Function(T param, void Function(String) report) task,
    T param,
    void Function(String message) onProgress,
  ) async {
    final receivePort = ReceivePort();
    await Isolate.spawn(
      _isolateEntryWithProgress<T, R>,
      _IsolateProgressPayload(task, param, receivePort.sendPort),
    );

    final completer = Completer<R>();
    receivePort.listen((message) {
      if (message is _IsolateProgress) {
        onProgress(message.message);
      } else if (message is _IsolateError) {
        completer.completeError(Exception(message.message));
        receivePort.close();
      } else if (message is _IsolateDone) {
        completer.complete(message.result as R);
        receivePort.close();
      }
    });
    return completer.future;
  }
}

class _IsolatePayload<T, R> {
  final IsolateTask<T, R> task;
  final T param;
  final SendPort sendPort;
  _IsolatePayload(this.task, this.param, this.sendPort);
}

class _IsolateProgressPayload<T, R> {
  final R Function(T, void Function(String)) task;
  final T param;
  final SendPort sendPort;
  _IsolateProgressPayload(this.task, this.param, this.sendPort);
}

class _IsolateProgress {
  final String message;
  _IsolateProgress(this.message);
}

class _IsolateDone {
  final dynamic result;
  _IsolateDone(this.result);
}

class _IsolateError {
  final String message;
  _IsolateError(this.message);
}

void _isolateEntry<T, R>(_IsolatePayload<T, R> payload) {
  try {
    final result = payload.task(payload.param);
    payload.sendPort.send(result);
  } catch (e) {
    payload.sendPort.send(_IsolateError(e.toString()));
  }
}

void _isolateEntryWithProgress<T, R>(_IsolateProgressPayload<T, R> payload) {
  try {
    final result = payload.task(payload.param, (msg) {
      payload.sendPort.send(_IsolateProgress(msg));
    });
    payload.sendPort.send(_IsolateDone(result));
  } catch (e) {
    payload.sendPort.send(_IsolateError(e.toString()));
  }
}
