class Test {
    constructor(data) {
        this._id = data.id;
        this._description = data.description;

        let buffer = [];
        for (var i in data.answers){
            buffer.push(data.answers[i]);
        }
        this._answers = buffer;

        if (data.image !== null) {
           this._image = data.image
        } else {
            this._image = null;
        }
    }
    get id() {
        return this._id;
    }
    get description() {
        return this._description;
    }
    get answers() {
        return this._answers;
    }
    get image(){
        return this._image;
    }
}

async function get_test() {
    let response = await fetch('https://127.0.0.1:5050/test');

    let data = await response.json();
    
    let test = new Test(data);
   // test.print_test();
  // alert(test.id);
    return test;
}

async function check_asnwer(test_id, answer_id) {
    let base_url = 'https://127.0.0.1:5050/check_answer';
    
    let qeury_data = URLSearchParams({
        test_id: test_id,
        answer_id: answer_id,
    });

    let query_data = qeury_data.toString();

    let full_url =  `${base_url}?${query_data}`;
    let response = await fetch(full_url);

    let data = await response.json();
    alert(data);
}



