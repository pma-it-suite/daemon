from flask import Flask, jsonify, request

app = Flask(__name__)


@app.route('/fetch', methods=['GET'])
def fetch_cmds():
    device_id = request.args.get('deviceId', None)

    if device_id is None:
        return jsonify({'error': 'deviceId parameter is required'}), 400

    # Return a simple JSON object
    return jsonify({'id': device_id, 'name': 'Command Name for ' + device_id})


@app.route('/ack', methods=['GET'])
def ack():
    command_id = request.args.get('commandId', None)

    if command_id is None:
        return jsonify({'error':
                        'commandId is required in the request body'}), 400

    # Return a 204 No Content response
    return '', 204


if __name__ == "__main__":
    app.run(host='127.0.0.1', port=4040)
