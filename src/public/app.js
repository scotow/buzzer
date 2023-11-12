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
    let { id, name } = await response.json();
    console.log(id, name);

    document.querySelector('.host.panel .title.panel > .labels > .label').innerText = name;

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
            case 'participantCount':
                document.querySelector('.host.panel .title.panel > .labels > .sub-label').innerText = `${data.count} participant${data.count !== 1 ? 's' : ''}`;
                break;
        }
    });
}

async function join() {
    document.body.classList.replace('lobby', 'participate');

    const response = await fetch('/rooms/id?name=dac', {
        method: 'GET',
    });
    let { id, name } = await response.json();
    console.log(id, name);

    document.querySelector('.participate.panel .title.panel > .labels > .label').innerText = name;

    const participateSocket = new WebSocket(`ws://localhost:8080/rooms/${id}/participate?name=Scotow`);
    participateSocket.addEventListener('message', (message) => {
        const data = JSON.parse(message.data);
        console.log(data);
        switch (data.event) {
            case 'participantCount':
                document.querySelector('.participate.panel .title.panel > .labels > .sub-label').innerText = `${data.count} participant${data.count !== 1 ? 's' : ''}`;
                break;
            case 'hostLeft':
                alert('The host has closed the room.');
                participateSocket.close();
                document.body.classList.replace('participate', 'lobby');
                break;
        }
    });

    document.querySelector('.participate.panel .buzzer').addEventListener('click', () => {
        participateSocket.send('buzz');
    });
}

// let shiftOffset = 72 / 2; // Placeholder.
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