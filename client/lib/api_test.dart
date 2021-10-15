import 'package:chopper/chopper.dart';
import 'package:smartx_whitedog/api/gen/example.swagger.dart';

void apiTest() async {
  final client = Example.create(ChopperClient(
    baseUrl: 'http://localhost:8000',
  ));
  final ret = await client.userGet();
  print(ret);
}
