// (async function() {
//     document.querySelector('.host').addEventListener('click', () => {
//         (async function() {
//             const response = await fetch("/rooms", {
//                 method: "POST",
//                 headers: {
//                     "Content-Type": "application/json",
//                 },
//                 body: JSON.stringify({
//                     name: "dac"
//                 }),
//             });
//             let id = await response.text();
//             console.log(id);
//
//             const hostSocket = new WebSocket(`ws://localhost:8080/rooms/${id}/host`);
//             hostSocket.addEventListener("message", (message) => {
//                 console.log(message.data);
//             });
//         })();
//     });
//
//     document.querySelector('.join').addEventListener('click', () => {
//         (async function() {
//             const response = await fetch('/rooms/id?name=dac', {
//                 method: 'GET',
//             });
//             let id = await response.text();
//             console.log(id);
//
//             const participateSocket = new WebSocket(`ws://localhost:8080/rooms/${id}/participate?name=Scotow`);
//             participateSocket.addEventListener("message", (message) => {
//                 console.log(message.data);
//             });
//
//             document.getElementById('buzzer').addEventListener('click', () => {
//                 participateSocket.send('buzz');
//             });
//         })();
//     });
//
// })().then(out => {
//     console.log(out);
// });

async function host() {
    document.body.classList.replace('lobby', 'host');

    const response = await fetch("/rooms", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            name: "dac"
        }),
    });
    let id = await response.text();
    console.log(id);

    const hostSocket = new WebSocket(`ws://localhost:8080/rooms/${id}/host`);
    hostSocket.addEventListener("message", (message) => {
        const data = JSON.parse(message.data);
        console.log(data);
        switch (data.event) {
            case 'buzzed':
                const buzzElem = document.createElement('div');
                buzzElem.classList.add('buzz');

                const usernameElem = document.createElement('div');
                usernameElem.classList.add('username');
                usernameElem.innerText = data.name;
                buzzElem.append(usernameElem);

                if (data.timestampDiff !== null) {
                    const timestampElem = document.createElement('div');
                    timestampElem.classList.add('timestamp');
                    if (data.timestampDiff < 1000) {
                        timestampElem.innerText = `+${data.timestampDiff}ms`;
                    } else if (data.timestampDiff < 10000) {
                        timestampElem.innerText = `+${Math.floor(data.timestampDiff / 100) / 10}s`;
                    } else {
                        timestampElem.innerText = `+${Math.floor(data.timestampDiff / 1000)}s`;
                    }
                    buzzElem.append(timestampElem);
                }

                document.querySelector('.host.panel .inner.panel').append(buzzElem);
                break;
        }
    });
}

async function join() {
    document.body.classList.replace('lobby', 'participate');

    const response = await fetch('/rooms/id?name=dac', {
        method: 'GET',
    });
    let id = await response.text();
    console.log(id);

    const participateSocket = new WebSocket(`ws://localhost:8080/rooms/${id}/participate?name=Scotow`);
    participateSocket.addEventListener("message", (message) => {
        console.log(message.data);
    });

    document.querySelector('.participate.panel .buzzer').addEventListener('click', () => {
        participateSocket.send('buzz');
    });
}

let shiftOffset = 72 / 2; // Placeholder.
// document.addEventListener('DOMContentLoaded', () => {
//     setTimeout(() => {
//         shiftOffset = (document.querySelector('#lobby .input.username').clientHeight + 14) / 2;
//     }, 250);
// });

document.querySelectorAll('.lobby.panel .mode').forEach((elem) => {
     elem.addEventListener('click', () => {
         if (elem.classList.contains('selected')) {
             return;
         }
         document.querySelector('.lobby.panel .mode.selected').classList.remove('selected');

         elem.classList.add('selected');
         document.querySelector('.lobby.panel .input.username').style.display = elem.classList.contains('host') ? 'none' : 'block';
         document.querySelector('.lobby.panel .action').innerText = `${elem.classList.contains('host') ? 'Create' : 'Join'} room`;
         // document.querySelector('#lobby > .panel').style.transform = `translate(-50%, calc(-50% - ${(elem.classList.contains('host') ? shiftOffset : 0) + 'px'}))`;
     })
});

// TODO: replace with submit event handling.
document.querySelector('.lobby.panel .action').addEventListener('click', () => {
    (async function() {
        if (document.querySelector('.lobby.panel .mode.selected').classList.contains('host')) {
            await host();
        } else {
            await join();
        }
    })();
});