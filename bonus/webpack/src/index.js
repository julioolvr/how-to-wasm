import loadAdd from '../add/src/lib.rs';

loadAdd().then(result => {
  const add = result.instance.exports.add;
  console.log(add(2, 3));
});
