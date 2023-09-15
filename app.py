from flask import Flask, jsonify

app = Flask(__name__)


@app.route("/")
def check():
    return jsonify(None), 200


if __name__ == '__main__':
    app.run(port=8181, debug=True)
