'use strict';

const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

const PROTO_PATH = __dirname + '/../proto/chat.proto';

const packageDefinition = protoLoader.loadSync(
    PROTO_PATH, {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true
    });

const chatModule = grpc.loadPackageDefinition(packageDefinition).chat;
const client = new chatModule.Chat('localhost:10000', grpc.credentials.createInsecure());

const joinResponse = {
    Denied: chatModule.JoinResponse.type.value[0].name,
    Accepted: chatModule.JoinResponse.type.value[1].name,
}

function join(member) {
    return new Promise((res, rej) => {
        client.join(member, (err, result) => {
            if (err) rej(err);
            if (result.response === joinResponse.Denied) rej(new Error('Access denied'));
            res({ token: result.token });
        });
    });
}

function send(newChatMessage) {
    return new Promise((res, rej) => {
        client.commit(newChatMessage, (err, result) => {
            if (err) rej(err);
            res(result);
        });
    });
}

function messages(after) {
    return new Promise((res, rej) => {
        const messageStream = client.chatLog(after);
        const messages = []
        messageStream.on('data', (message) => {
            messages.push(message);
        });
        messageStream.on('end', () => res(messages));
        messageStream.on('close', () => rej(new Error('Message read stream closed')));
        messageStream.on('error', err => rej(err));
    });
}

module.exports = {
    join,
    send,
    messages,
}